use std::iter::Map;

use engine_ecs::EntityID;
use engine_num::Vec3;
use engine_registry::{BuildingKind, Registry};
use godot::{
    builtin::Variant,
    engine::{load, PackedScene},
    prelude::{
        meta::VariantMetadata, Array, Basis, FromVariant, Gd, GodotClass, Inherits, Node,
        SceneTree, StringName, Vector3,
    },
};

use crate::{
    netman::NetmanVariant,
    universe::{rotations::CompactBasis, tilemap::TilePos},
};

pub trait ToGodot {
    type Output;
    fn to_godot(&self) -> Self::Output;
}

pub trait FromGodot<T> {
    fn from_godot(val: T) -> Self;
}

impl ToGodot for Vec3 {
    type Output = Vector3;

    fn to_godot(&self) -> Self::Output {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl FromGodot<Vector3> for Vec3 {
    fn from_godot(val: Vector3) -> Self {
        Vec3::new(val.x, val.y, val.z)
    }
}

impl FromGodot<Vector3> for TilePos {
    fn from_godot(val: Vector3) -> Self {
        Self {
            x: (val.x / Self::GRID_STEP).round() as i32,
            y: (val.y / Self::GRID_STEP).round() as i32,
            z: (val.z / Self::GRID_STEP).round() as i32,
        }
    }
}

impl ToGodot for TilePos {
    type Output = Vector3;

    fn to_godot(&self) -> Self::Output {
        Vector3 {
            x: self.x as f32 * Self::GRID_STEP,
            y: self.y as f32 * Self::GRID_STEP,
            z: self.z as f32 * Self::GRID_STEP,
        }
    }
}

impl ToGodot for [f32; 3] {
    type Output = Vector3;

    fn to_godot(&self) -> Self::Output {
        Vector3::new(self[0], self[1], self[2])
    }
}

impl ToGodot for CompactBasis {
    type Output = Basis;

    fn to_godot(&self) -> Self::Output {
        let mut raw_basis = [[0.0f32; 3]; 3];
        for (i, &e) in self.0.iter().enumerate() {
            if e > 0 {
                raw_basis[i][e as usize - 1] = 1.0;
            } else {
                raw_basis[i][(-e) as usize - 1] = -1.0;
            }
        }
        Basis::from_cols(
            raw_basis[0].to_godot(),
            raw_basis[1].to_godot(),
            raw_basis[2].to_godot(),
        )
    }
}

impl FromGodot<Variant> for EntityID {
    fn from_godot(val: Variant) -> Self {
        EntityID::from_raw(val.to::<i64>() as u64) // Reinterpret u64 <> i64, because godot can't store u64 directly
    }
}

impl ToGodot for EntityID {
    type Output = Variant;
    fn to_godot(&self) -> Self::Output {
        Variant::from(self.to_raw() as i64)
    }
}

pub struct ArrayIter<T: VariantMetadata> {
    array: Array<T>,
    pointer: usize,
}

impl<T: VariantMetadata> ArrayIter<T> {
    pub fn new(array: Array<T>) -> Self {
        Self { array, pointer: 0 }
    }
}

impl<T: VariantMetadata + FromVariant> Iterator for ArrayIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pointer < self.array.len() {
            let ret = self.array.get(self.pointer);
            self.pointer += 1;
            Some(ret)
        } else {
            None
        }
    }
}

pub trait SceneTreeExt {
    type RetIterator<Derived: GodotClass>: Iterator<Item = Gd<Derived>>;
    fn iter_group<Derived>(
        &mut self,
        group_name: impl Into<StringName>,
    ) -> Self::RetIterator<Derived>
    where
        Derived: GodotClass + Inherits<Node>;
}

impl SceneTreeExt for SceneTree {
    type RetIterator<Derived: GodotClass> = Map<ArrayIter<Gd<Node>>, fn(Gd<Node>) -> Gd<Derived>>;
    fn iter_group<Derived>(
        &mut self,
        group_name: impl Into<StringName>,
    ) -> Self::RetIterator<Derived>
    where
        Derived: GodotClass + Inherits<Node>,
    {
        let group = self.get_nodes_in_group(group_name.into());
        ArrayIter::new(group).map(Gd::<Node>::cast::<Derived>)
    }
}

pub trait OptionNetmanExt {
    fn get(&self) -> &NetmanVariant;
    fn get_mut(&mut self) -> &mut NetmanVariant;
}

impl OptionNetmanExt for Option<NetmanVariant> {
    fn get(&self) -> &NetmanVariant {
        self.as_ref().unwrap()
    }

    fn get_mut(&mut self) -> &mut NetmanVariant {
        self.as_mut().unwrap()
    }
}

pub trait RegistryExt {
    fn scene_by_building_kind(&self, kind: BuildingKind) -> Gd<PackedScene>;
    fn scene_by_building_index(&self, index: usize) -> Gd<PackedScene>;
}

impl RegistryExt for Registry {
    fn scene_by_building_kind(&self, kind: BuildingKind) -> Gd<PackedScene> {
        let device_name = self
            .building_by_kind(kind)
            .map(|x| x.name)
            .unwrap_or("dummy");
        load::<PackedScene>(format!("vessel/buildings/{device_name}.tscn"))
    }
    fn scene_by_building_index(&self, index: usize) -> Gd<PackedScene> {
        let device_name = self.buildings.get(index).map(|x| x.name).unwrap_or("dummy");
        load::<PackedScene>(format!("vessel/buildings/{device_name}.tscn"))
    }
}
