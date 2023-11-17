use super::internal::ComponentStorageProvider;
use super::ArchetypeID;
use super::LocalTypeIndex;
use crate::internal::DynDispath;
use crate::TypeIndex;
use crate::World;
use smallvec::SmallVec;
use std::marker::PhantomData;

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
    fn add_to_archetype_in_storage(self, world: &mut World<Storage>, archetype: ArchetypeID);
}

impl<Storage> BundleTrait<Storage> for () {
    fn type_ids() -> TypeIndexStorage {
        TypeIndexStorage::new()
    }

    fn add_to_archetype_in_storage(self, world: &mut World<Storage>, archetype: ArchetypeID) {}
}

pub struct Bundle<B0, B1, Storage>(pub B0, pub B1, pub PhantomData<fn() -> Storage>);

impl<Storage, T> BundleTrait<Storage> for T
where
    Storage: ComponentStorageProvider<T> + Default + Clone + DynDispath,
    T: Into<Bundle<T, (), Storage>> + LocalTypeIndex<Storage>,
{
    fn type_ids() -> TypeIndexStorage {
        TypeIndexStorage::from_elem(T::TYPE_INDEX, 1)
    }

    fn add_to_archetype_in_storage(self, world: &mut World<Storage>, archetype: ArchetypeID) {
        world.add_bundle_to_archetype(archetype, self)
    }
}

impl<Storage, B0, B1> BundleTrait<Storage> for Bundle<B0, B1, Storage>
where
    B0: BundleTrait<Storage>,
    B1: BundleTrait<Storage>,
{
    fn type_ids() -> TypeIndexStorage {
        let mut indexes = B0::type_ids();
        indexes.extend_from_slice(&B1::type_ids());
        indexes
    }

    fn add_to_archetype_in_storage(self, world: &mut World<Storage>, archetype: ArchetypeID) {
        self.0.add_to_archetype_in_storage(world, archetype);
        self.1.add_to_archetype_in_storage(world, archetype);
    }
}

impl<Storage, B0> From<(B0,)> for Bundle<B0, (), Storage>
where
    B0: BundleTrait<Storage>,
{
    fn from(value: (B0,)) -> Self {
        Bundle(value.0, (), PhantomData)
    }
}

impl<Storage, B0, B1> From<(B0, B1)> for Bundle<B0, B1, Storage>
where
    B0: BundleTrait<Storage>,
    B1: BundleTrait<Storage>,
{
    fn from(value: (B0, B1)) -> Self {
        Bundle(value.0, value.1, PhantomData)
    }
}

impl<Storage, B0, B1, B2> From<(B0, B1, B2)> for Bundle<Bundle<B0, B1, Storage>, B2, Storage>
where
    B0: BundleTrait<Storage>,
    B1: BundleTrait<Storage>,
    B2: BundleTrait<Storage>,
{
    fn from(value: (B0, B1, B2)) -> Self {
        Bundle(Bundle(value.0, value.1, PhantomData), value.2, PhantomData)
    }
}
