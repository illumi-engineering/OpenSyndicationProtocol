/// # Data Type Registry
///
/// The data type registry is responsible for:
///
/// - Holding instances of [`DataMarshaller`] and associating them to their [`Data`]
///   type by using [`TypeId`].
/// - Associating their type [Uuid] with their [`TypeId`]. (for info, see
///   [`Data::get_id_static`])

use std::any::TypeId;
use std::collections::HashMap;


use uuid::Uuid;

use crate::{Data, DataType};

/// A registry for holding types of [Data] and their [`DataMarshaller`], and
/// associating their type [`Uuid`] with their [`TypeId`]
#[allow(clippy::module_name_repetitions)]
pub struct DataTypeRegistry {
    types: HashMap<TypeId, DataType<dyn Data>>,
    id_map: HashMap<Uuid, TypeId>,
}

impl DataTypeRegistry {
    #[must_use] pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    /// Register [`TData`] with its own [`DataMarshaller`] and associate its [`Uuid`]
    /// with its [`TypeId`]
    pub fn register<TData>(&mut self)
    where
        TData : Data + 'static
    {
        let type_id = TypeId::of::<TData>();
        // SAFETY: Mapping by `TypeId` ensures we have the proper type
        #[allow(clippy::transmute_undefined_repr)]
        self.types.insert(type_id, unsafe { std::mem::transmute::<DataType<TData>, DataType<dyn Data>>(DataType::<TData>::new()) });
        self.id_map.insert(TData::get_id_static(), type_id);
    }

    /// Get a marshaller for [`TData`] based on its [`TypeId`]
    #[must_use] pub fn by_type_id<TData>(&self) -> Option<&DataType<TData>>
    where
        TData : Data + 'static,
    {
        // SAFETY: Mapping by `TypeId` ensures we have the proper type
        unsafe { std::mem::transmute(self.types.get(&TypeId::of::<TData>())) }
    }

    /// Get the marshaller for [`TData`] based on a type [`Uuid`]
    #[must_use] pub fn by_uuid(&self, uuid: &Uuid) -> Option<&DataType<dyn Data>>
    where
    {
        let type_id = self.id_map.get(uuid)?;
        self.types.get(type_id)
    }

    /// Check whether a data type is registered based on its [`Uuid`]
    #[must_use] pub fn has_by_uuid(&self, uuid: &Uuid) -> bool {
        self.id_map.contains_key(uuid)
    }
}

impl Default for DataTypeRegistry {
    fn default() -> Self {
        Self::new()
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
    fn test_registry_fetch_by_type() {
        let mut registry = DataTypeRegistry::new();

        registry.register::<MyData>();

        let got = registry.by_type_id::<MyData>();
        assert!(got.is_some());
        assert_eq!(got.expect("Anya Forger is best :3").get_id(), MyData::get_id_static());
    }

    #[test]
    fn test_registry_fetch_by_uuid() {
        let mut registry = DataTypeRegistry::new();

        registry.register::<MyData>();

        let got = registry.by_uuid(&MyData::get_id_static());
        assert!(got.is_some());
    }
}