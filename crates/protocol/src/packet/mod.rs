use std::io::ErrorKind;
use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};

use tokio::io;
use tokio_util::codec::{Decoder, Encoder};

use uuid::Uuid;

pub mod handshake;
pub mod transfer;
pub mod data;

/// The maximum length a packet can be. Any data that needs to be sent and is
/// longer than this maximum should be chunked into multiple packets.
pub const PACKET_MAX_LENGTH: usize = 8 * 1024 * 1024;

/// This trait is used to serialize from a packet to a [`BytesMut`]
#[allow(clippy::module_name_repetitions)]
pub trait SerializePacket {
    /// Serialize to a [`BytesMut`]
    ///
    /// # Errors
    /// If serialization fails
    fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize>;

    /// Write a `String` to `buf` and return how many bytes were written.
    fn write_string(&self, buf: &mut BytesMut, string: &String) -> usize where Self : Sized {
        let bytes = string.as_bytes();
        buf.put_u64(bytes.len() as u64);
        buf.put_slice(bytes);
        8 + bytes.len() // u64 = 8 bytes
    }

    /// Write an `Option<String>` to `buf` and return how many bytes were
    /// written.
    fn write_optional_string(&self, buf: &mut BytesMut, string: &Option<String>) -> usize where Self: Sized {
        buf.put_u8(u8::from(string.is_some()));
        let mut bytes_written = 1;
        if let Some(str) = string {
            bytes_written += self.write_string(buf, str);
        }
        bytes_written
    }

    /// Write a `Uuid` to `buf` and return how many bytes were written. This
    /// function should always return `16` as it writes the `uuid` as a `u128`
    /// directly to the buffer.
    fn write_uuid(&self, buf: &mut BytesMut, uuid: &Uuid) -> usize where Self: Sized {
        buf.put_u128(uuid.as_u128());
        16 // u128 is 16 bytes
    }

    /// Write an `Option<Uuid>` to `buf` and return how many bytes were written.
    fn write_optional_uuid(&self, buf: &mut BytesMut, uuid: &Option<Uuid>) -> usize where Self: Sized {
        buf.put_u8(u8::from(uuid.is_some()));
        let mut bytes_written = 8;
        if let Some(id) = uuid {
            bytes_written += self.write_uuid(buf, id);
        }
        bytes_written
    }
}

/// Trait for a packet that can be deserialized from a [`BytesMut`].
#[allow(clippy::module_name_repetitions)]
pub trait DeserializePacket {
    /// The type that this deserializes to
    type Output;

    /// Deserialize from a [`BytesMut`]
    ///
    /// # Errors
    /// If Deserialization fails.
    fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output>;

    /// From a given [`BytesMut`], read the next length (u16) and extract the
    /// string bytes, returning a [String].
    ///
    /// # Errors
    /// Returns [`io::ErrorKind::InvalidData`] if the string is not valid UTF-8
    ///
    /// # Panics
    /// If the serialized length it decodes is invalid
    fn read_string(buf: &mut BytesMut) -> io::Result<String> {
        let length = buf.get_u64();

        // Given the length of our string, only read in that quantity of bytes
        let mut bytes = vec![0u8; usize::try_from(length).map_err(|e| { io::Error::new(ErrorKind::InvalidData, e.to_string()) })?];
        buf.copy_to_slice(&mut bytes);

        // And attempt to decode it as UTF8
        String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid utf8"))
    }

    /// Read an `Option<String>` from `buf`
    ///
    /// # Errors
    /// If the inner [`Self::read_string`] errors
    fn read_optional_string(buf: &mut BytesMut) -> io::Result<Option<String>> {
        Ok(if buf.get_u8() != 0 { // if the boolean is set read the optional value
            Some(Self::read_string(buf)?)
        } else { None })
    }

    /// Read a `Uuid` from `buf`
    fn read_uuid(buf: &mut BytesMut) -> Uuid {
        Uuid::from_u128(buf.get_u128())
    }

    /// Read an `Option<Uuid>` from `buf`
    fn read_optional_uuid(buf: &mut BytesMut) -> Option<Uuid> {
        if buf.get_u8() != 0 { // if the boolean is set read the optional value
            Some(Self::read_uuid(buf))
        } else { None }
    }
}

/// A tokio codec for deserializing packets that implement [`DeserializePacket`]
/// in a [`FramedRead`]. For more information see [`tokio_util::codec`].
///
/// [`FramedRead`]: tokio_util::codec::FramedRead
#[allow(clippy::module_name_repetitions)]
pub struct PacketDecoder<PacketType: DeserializePacket> {
    _packet_type: PhantomData<PacketType>
}

impl<PacketType : DeserializePacket> PacketDecoder<PacketType> {
    #[must_use] pub const fn new() -> Self {
        Self {
            _packet_type: PhantomData,
        }
    }
}

impl<PacketType : DeserializePacket> Default for PacketDecoder<PacketType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PacketType: DeserializePacket> Decoder for PacketDecoder<PacketType> {
    type Item = PacketType::Output;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > PACKET_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {length} is too large.")
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let packet = PacketType::deserialize(&mut BytesMut::from(data.as_slice()))?;

        Ok(Some(packet))
    }
}

/// A tokio codec for serializing packets that implement [`SerializePacket`]
/// in a [`FramedWrite`]. For more information see [`tokio_util::codec`].
///
/// [`FramedWrite`]: tokio_util::codec::FramedWrite
#[allow(clippy::module_name_repetitions)]
pub struct PacketEncoder<PacketType : SerializePacket> {
    _packet_type: PhantomData<PacketType>,
}

impl<PacketType : SerializePacket> PacketEncoder<PacketType> {
    #[must_use] pub const fn new() -> Self {
        Self {
            _packet_type: PhantomData,
        }
    }
}

impl<PacketType : SerializePacket> Default for PacketEncoder<PacketType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PacketType: SerializePacket> Encoder<PacketType> for PacketEncoder<PacketType> {
    type Error = io::Error;

    fn encode(&mut self, item: PacketType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = &mut BytesMut::with_capacity(PACKET_MAX_LENGTH);
        item.serialize(buf)?;

        if buf.len() > PACKET_MAX_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", buf.len())
            ));
        }

        // Convert the length into a byte array.
        // The cast to u32 cannot overflow due to the length check above.
        #[allow(clippy::cast_possible_truncation)]
        let len_slice = u32::to_le_bytes(buf.len() as u32);

        // Reserve space in the buffer.
        dst.reserve(4 + buf.len());

        // Write the length and string to the buffer.
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::io;
    use bytes::{Buf, BufMut, BytesMut};
    use uuid::Uuid;
    use crate::packet::{DeserializePacket, PACKET_MAX_LENGTH, SerializePacket};

    /// A basic test packet for validating basic serialization and
    /// deserialization of values that implement [`SerializePacket`] and
    /// [`DeserializePacket`].
    #[derive(PartialEq, Debug)]
    struct TestPacket {
        bool: bool,
        int: u8,
        string: String,
    }

    impl SerializePacket for TestPacket {
        fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
            buf.put_u8(u8::from(self.bool));
            let mut bytes_written = 1;
            buf.put_u8(self.int);
            bytes_written += 1;

            bytes_written += self.write_string(buf, &self.string);
            Ok(bytes_written)
        }
    }

    impl DeserializePacket for TestPacket {
        type Output = Self;

        fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
            Ok(Self {
                bool: buf.get_u8() != 0,
                int: buf.get_u8(),
                string: Self::read_string(buf)?,
            })
        }
    }

    /// A test packet for testing serialization/deserialization of [Uuid].
    #[derive(PartialEq, Debug)]
    struct TestUuidPacket {
        uuid: Uuid,
    }

    impl SerializePacket for TestUuidPacket {
        fn serialize(&self, buf: &mut BytesMut) -> io::Result<usize> {
            let bytes_written = self.write_uuid(buf, &self.uuid);
            Ok(bytes_written)
        }
    }

    impl DeserializePacket for TestUuidPacket {
        type Output = Self;

        fn deserialize(buf: &mut BytesMut) -> io::Result<Self::Output> {
            Ok(Self {
                uuid: Self::read_uuid(buf),
            })
        }
    }

    /// Create a test packet whose values correspond to [`TEST_PACKET_BYTES`]
    fn create_test_packet() -> TestPacket {
        TestPacket {
            bool: true,
            int: 32u8,
            string: String::from("hello"),
        }
    }

    /// A raw byte representation of the packet [`create_test_packet`] creates
    //                                    bool: true
    //                                    |    int: 32u8
    //                                    |    |     [string: "hello"]
    //                                    |    |     [len (usize->u64) data == "hello"]
    //                                    |    |      |                |
    //                                    V    V      V                V
    const TEST_PACKET_BYTES: &[u8; 15] = &[1u8, 32u8, 0,0,0,0,0,0,0,5, 104,101,108,108,111];

    /// This test checks two things:
    /// - Whether the data serialized from a [`SerializePacket`] is accurate on
    ///   the resulting buffer
    /// - Whether the serialize function returns the correct amount of bytes
    ///   written
    #[test]
    fn test_basic_serialization() -> io::Result<()> {
        let buf = &mut BytesMut::new();

        let bytes_written = create_test_packet().serialize(buf)?;

        assert_eq!(&buf[..], TEST_PACKET_BYTES);
        assert_eq!(bytes_written, buf.len());
        Ok(())
    }

    /// This test only checks whether data deserialized from the buffer matches
    /// the expected value of the [`TestPacket`]
    #[test]
    fn test_basic_deserialization() -> io::Result<()> {
        let buf = &mut BytesMut::from(TEST_PACKET_BYTES.as_slice());
        let packet = TestPacket::deserialize(buf)?;

        assert_eq!(packet, create_test_packet());
        Ok(())
    }

    #[test]
    fn test_uuid_serde() -> io::Result<()> {
        let buf = &mut BytesMut::with_capacity(PACKET_MAX_LENGTH);
        let uuid = Uuid::new_v4();
        let packet = TestUuidPacket {
            uuid,
        };

        // serialize the packet
        let bytes_written = packet.serialize(buf)?;
        assert_eq!(bytes_written, buf.len());

        // deserialize the packet
        let packet_de = TestUuidPacket::deserialize(buf)?;
        assert_eq!(packet, packet_de);

        Ok(())
    }
}