use std::marker::PhantomData;

use super::ArchetypeID;

use crate::TypeIndex;

use smallvec::SmallVec;

use super::internal::ComponentStorageProvider;

use super::LocalTypeIndex;

pub trait Component<Storage>: LocalTypeIndex<Storage> {}

impl<Storage, T> Component<Storage> for T
where
    Storage: ComponentStorageProvider<T>,
    T: LocalTypeIndex<Storage>,
{
}

pub(crate) type TypeIndexStorage = SmallVec<[TypeIndex; 8]>;

pub trait BundleTrait<Storage> {
    fn type_ids() -> TypeIndexStorage;
    fn add_to_archetype_in_storage(self, storage: &mut Storage, archetype: ArchetypeID);
}

impl<Storage> BundleTrait<Storage> for () {
    fn type_ids() -> TypeIndexStorage {
        TypeIndexStorage::new()
    }

    fn add_to_archetype_in_storage(self, _storage: &mut Storage, _archetype: ArchetypeID) {}
}

pub(crate) struct Bundle<B0, B1, Storage>(B0, B1, PhantomData<fn() -> Storage>);

impl<Storage, T> BundleTrait<Storage> for T
where
    Storage: ComponentStorageProvider<T>,
    T: Into<Bundle<T, (), Storage>> + LocalTypeIndex<Storage>,
{
    fn type_ids() -> TypeIndexStorage {
        TypeIndexStorage::from_elem(T::TYPE_INDEX, 1)
    }

    fn add_to_archetype_in_storage(self, storage: &mut Storage, archetype: ArchetypeID) {
        storage.storage_mut().add_to_archetype(archetype, self)
    }
}

impl<Storage, B0: BundleTrait<Storage>, B1: BundleTrait<Storage>> BundleTrait<Storage>
    for Bundle<B0, B1, Storage>
{
    fn type_ids() -> TypeIndexStorage {
        let mut indexes = B0::type_ids();
        indexes.extend_from_slice(&B1::type_ids());
        indexes
    }

    fn add_to_archetype_in_storage(self, storage: &mut Storage, archetype: ArchetypeID) {
        self.0.add_to_archetype_in_storage(storage, archetype);
        self.1.add_to_archetype_in_storage(storage, archetype);
    }
}

impl<Storage, B0: BundleTrait<Storage>> From<(B0,)> for Bundle<B0, (), Storage> {
    fn from(value: (B0,)) -> Self {
        Bundle(value.0, (), PhantomData)
    }
}

impl<Storage, B0: BundleTrait<Storage>, B1: BundleTrait<Storage>> From<(B0, B1)>
    for Bundle<B0, B1, Storage>
{
    fn from(value: (B0, B1)) -> Self {
        Bundle(value.0, value.1, PhantomData)
    }
}

impl<Storage, B0: BundleTrait<Storage>, B1: BundleTrait<Storage>, B2: BundleTrait<Storage>>
    From<(B0, B1, B2)> for Bundle<Bundle<B0, B1, Storage>, B2, Storage>
{
    fn from(value: (B0, B1, B2)) -> Self {
        Bundle(Bundle(value.0, value.1, PhantomData), value.2, PhantomData)
    }
}
