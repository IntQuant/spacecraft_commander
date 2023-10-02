use engine_num::Vec3;
use godot::prelude::Vector3;

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
