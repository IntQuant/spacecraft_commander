use std::{iter::Map, ops::FnMut};

use engine_num::Vec3;
use godot::prelude::{
    meta::VariantMetadata, Array, FromVariant, Gd, GodotClass, Inherits, Node, SceneTree,
    StringName, Vector3,
};

pub trait IntoGodot {
    type Output;
    fn into_godot(&self) -> Self::Output;
}

pub trait FromGodot<T> {
    fn from_godot(val: T) -> Self;
}

impl IntoGodot for Vec3 {
    type Output = Vector3;

    fn into_godot(&self) -> Self::Output {
        Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}

impl FromGodot<Vector3> for Vec3 {
    fn from_godot(val: Vector3) -> Self {
        Vec3::new(val.x.into(), val.y.into(), val.z.into())
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

pub(crate) trait SceneTreeExt {
    type RetIterator<Derived: GodotClass, F: FnMut(Gd<Node>) -> Gd<Derived>>: Iterator<
        Item = Gd<Derived>,
    >;
    fn iter_group<Derived>(
        &mut self,
        group_name: impl Into<StringName>,
    ) -> Self::RetIterator<Derived, fn(Gd<Node>) -> Gd<Derived>>
    where
        Derived: GodotClass + Inherits<Node>;
}

fn cast<Derived: GodotClass + Inherits<Node>>(x: Gd<Node>) -> Gd<Derived> {
    x.cast::<Derived>()
}

impl SceneTreeExt for SceneTree {
    type RetIterator<Derived: GodotClass, F: FnMut(Gd<Node>) -> Gd<Derived>> =
        Map<ArrayIter<Gd<Node>>, F>;
    fn iter_group<Derived>(
        &mut self,
        group_name: impl Into<StringName>,
    ) -> Self::RetIterator<Derived, fn(Gd<Node>) -> Gd<Derived>>
    where
        Derived: GodotClass + Inherits<Node>,
    {
        let group = self.get_nodes_in_group(group_name.into());
        ArrayIter::new(group).map(cast)
    }
}
