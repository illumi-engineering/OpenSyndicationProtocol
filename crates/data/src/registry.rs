use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::Unsize;
use std::ops::Deref;
use downcast_rs::Downcast;
use uuid::Uuid;
use crate::{Data, DataMarshaller};

pub struct DataTypeRegistry {
    items: HashMap<TypeId, Box<dyn DataMarshaller<DataType=dyn Data>>>,
    id_map: HashMap<Uuid, TypeId>,
}

impl DataTypeRegistry {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    pub fn register<TData : Data + 'static, TMarshaller : DataMarshaller<DataType = (dyn Data + 'static)> + 'static>(&mut self, uuid: Uuid, data_type: TMarshaller) {
        let type_id = TypeId::of::<TData>();
        self.items.insert(type_id, Box::new(data_type));
        self.id_map.insert(uuid, type_id);
    }

    pub fn get_codec_by_type_id<TData : Data + 'static, TMarshaller : DataMarshaller + Sized + 'static>(&self) -> Option<&TMarshaller> {
        let marshaller = self.items.get(&TypeId::of::<TData>())?;
        marshaller.as_any().downcast_ref::<TMarshaller>()
    }

    pub fn get_codec_by_uuid<TData : Data + 'static, TMarshaller : DataMarshaller + Sized + 'static>(&self, uuid: &Uuid) -> Option<&TMarshaller> {
        let type_id = self.id_map.get(uuid)?;
        let marshaller = self.items.get(type_id)?;
        marshaller.as_any().downcast_ref::<TMarshaller>()
    }
}

#[cfg(test)]
mod tests {
    #![feature(unsize)]
    use std::str::FromStr;
    use bincode::{Decode, Encode};
    use uuid::Uuid;
    use crate::{Data, DataMarshaller, registry::DataTypeRegistry};

    #[derive(Encode, Decode)]
    struct MyData {
        test_int: u8,
    }

    impl Data for MyData {}

    struct MyDataMarshaller {}

    impl MyDataMarshaller {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl DataMarshaller for MyDataMarshaller {
        type DataType = MyData;

        fn get_id_static() -> Uuid
        where
            Self: Sized
        {
            Uuid::from_str("1e563afd-a570-47ac-bd8a-131f9d3cad79").unwrap()
        }
    }

    fn test_registry() {
        let mut registry = DataTypeRegistry::new();

        let marshaller = MyDataMarshaller::new();

        registry.register::<MyData, MyDataMarshaller>(MyDataMarshaller::get_id_static(), marshaller);


    }
}