use engine_num::Vec3;
use godot::prelude::{meta::VariantMetadata, Array, FromVariant, Vector3};

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
