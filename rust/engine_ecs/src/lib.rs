use slotmapd::new_key_type;

pub mod internal {

    pub trait ComponentStorageProvider<T> {
        fn storage_for(&mut self) -> &mut Vec<Vec<T>>;
    }
}

new_key_type! { pub struct EntityID; }
new_key_type! { struct ArchetypeID; }

struct EntityInfo {
    pub archetype_id: ArchetypeID,
    pub in_archetype_id: u32,
}

struct ArchetypeInfo {}

pub struct World<Storage> {
    entities: slotmapd::HopSlotMap<EntityID, EntityInfo>,
    archetypes: slotmapd::SlotMap<ArchetypeID, ArchetypeInfo>,
    storage: Storage,
}
