use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;

use downcast_rs::Downcast;

use uuid::Uuid;

use crate::{Data, DataMarshaller};

#[derive(Clone)]
pub struct DataTypeRegistry

{
    items: HashMap<TypeId, DataMarshaller<Box<dyn Data + 'static>>>,
    id_map: HashMap<Uuid, TypeId>,
}

impl<TData> DataTypeRegistry
where
    TData : Data + 'static + Clone,
    Box<TData>: Data,
{
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    pub fn register<TData>(&mut self, marshaller: DataMarshaller<Box<TData>>)
    {
        let type_id = TypeId::of::<TData>();
        self.items.insert(type_id, marshaller);
        self.id_map.insert(TData::get_id_static(), type_id);
    }

    pub fn get_codec_by_type_id<TData>(&self) -> Option<&DataMarshaller<Box<TData>>>
    where
        TData : Data + 'static + Clone,
    {
        self.items.get(&TypeId::of::<TData>())
    }

    pub fn get_codec_by_uuid<TData>(&self, uuid: &Uuid) -> Option<&DataMarshaller<Box<TData>>>
    where
        TData : Data + 'static + Clone,
    {
        let type_id = self.id_map.get(uuid)?;
        self.items.get(type_id)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use bincode::{Decode, Encode};
    use uuid::Uuid;
    use crate::{Data, DataMarshaller, impl_data, registry::DataTypeRegistry};

    #[derive(Encode, Decode, Clone)]
    struct MyData {
        test_int: u8,
    }

    impl_data!(MyData, "9eddbf56-8cba-4962-9769-dcc84f1eefae");

    #[test]
    fn test_registry() {
        let mut registry = DataTypeRegistry::new();

        registry.register::<MyData>(DataMarshaller::new());

        let got = registry.get_codec_by_type_id::<MyData>();
        assert!(got.is_some());
        assert_eq!(got.unwrap().get_id(), MyData::get_id_static());
    }
}