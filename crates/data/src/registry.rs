/// # Data Type Registry
///
/// The data type registry is responsible for:
///
/// - Holding instances of [DataMarshaller] and associating them to their [Data]
///   type by using [TypeId].
/// - Associating their type [Uuid] with their [TypeId]. (for info, see
///   [Data::get_id_static])

use std::any::{Any, TypeId};
use std::boxed::ThinBox;
use std::collections::HashMap;
use std::ops::Deref;

use downcast_rs::Downcast;

use uuid::Uuid;

use crate::{Data, DataMarshaller};

/// A registry for holding types of [Data] and their [DataMarshaller], and
/// associating their type [Uuid] with their [TypeId]
#[derive(Clone)]
pub struct DataTypeRegistry {
    handlers: HashMap<TypeId, ThinBox<dyn Any>>,
    marshallers: HashMap<TypeId, DataMarshaller>,
    id_map: HashMap<Uuid, TypeId>,
}

impl DataTypeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            marshallers: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    /// Register [TData] with its own [DataMarshaller] and associate its [Uuid]
    /// with its [TypeId]
    pub fn register<TData>(&mut self)
    where
        TData : Data + 'static
    {
        let type_id = TypeId::of::<TData>();
        self.marshallers.insert(type_id, DataMarshaller::new(TData::get_id_static()));
        self.id_map.insert(TData::get_id_static(), type_id);
    }

    /// Get a marshaller for [TData] based on its [TypeId]
    pub fn get_marshaller_by_type_id<TData>(&self) -> Option<&DataMarshaller>
    where
        TData : Data + 'static,
    {
        self.marshallers.get(&TypeId::of::<TData>())
    }

    /// Get the marshaller for [TData] based on a type [Uuid]
    pub fn get_marshaller_by_uuid<TData>(&self, uuid: &Uuid) -> Option<&DataMarshaller>
    where
        TData : Data + 'static,
    {
        let type_id = self.id_map.get(uuid)?;
        self.marshallers.get(type_id)
    }

    pub fn get_handler<T: ?Sized + 'static>(&self) -> Option<&T> {
        let any = self.handlers.get(&TypeId::of::<T>());
        any.map(|any| unsafe { std::mem::transmute::<_, &ThinBox<T>>(any) }.deref())
    }

    /// Check whether a data type is registered based on its [Uuid]
    pub fn has_data_type_by_uuid(&self, uuid: &Uuid) -> bool {
        self.id_map.contains_key(uuid)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use bincode::{Decode, Encode};
    use uuid::Uuid;
    use crate::{Data, impl_data, registry::DataTypeRegistry};

    #[derive(Encode, Decode, Clone)]
    struct MyData {
        test_int: u8,
    }

    impl_data!(MyData, "9eddbf56-8cba-4962-9769-dcc84f1eefae");

    #[test]
    fn test_registry() {
        let mut registry = DataTypeRegistry::new();

        registry.register::<MyData>();

        let got = registry.get_codec_by_type_id::<MyData>();
        assert!(got.is_some());
        assert_eq!(got.unwrap().get_id(), MyData::get_id_static());
    }
}