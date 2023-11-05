use std::iter::Map;

use engine_num::Vec3;
use godot::prelude::{
    meta::VariantMetadata, Array, Basis, FromVariant, Gd, GodotClass, Inherits, Node, SceneTree,
    StringName, Vector3,
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
        Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}

impl FromGodot<Vector3> for Vec3 {
    fn from_godot(val: Vector3) -> Self {
        Vec3::new(val.x.into(), val.y.into(), val.z.into())
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
