use super::internal::ComponentStorageProvider;
use super::ArchetypeID;
use super::LocalTypeIndex;
use crate::TypeIndex;
use crate::World;
use engine_macro::gen_bundle_tuple_impls;
use smallvec::SmallVec;

pub trait Component<Storage>: LocalTypeIndex<Storage> {}

impl<Storage, T> Component<Storage> for T
where
    Storage: ComponentStorageProvider<T>,
    T: LocalTypeIndex<Storage>,
{
}

pub type TypeIndexStorage = SmallVec<[TypeIndex; 8]>;

pub trait Bundle<Storage> {
    fn type_ids() -> TypeIndexStorage;
    fn add_to_archetype_in_storage(self, world: &mut World<Storage>, archetype: ArchetypeID);
}

impl<Storage> Bundle<Storage> for () {
    fn type_ids() -> TypeIndexStorage {
        TypeIndexStorage::new()
    }

    fn add_to_archetype_in_storage(self, _world: &mut World<Storage>, _archetype: ArchetypeID) {}
}

gen_bundle_tuple_impls!(1);
gen_bundle_tuple_impls!(2);
gen_bundle_tuple_impls!(3);
gen_bundle_tuple_impls!(4);
gen_bundle_tuple_impls!(5);
gen_bundle_tuple_impls!(6);
