use num_traits::{NumCast, ToPrimitive, Zero};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::*;

/// Fixed-point numbers.
///
/// Ensures that:
/// 1. Positions have the same precision at all points.
/// 2. Numerical ops have the same results on all platforms.
/// 3. More bits of precision can be used when necessary (e.g. i128 for position of spacecraft in a galaxy).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Fixed<T, const P: u8>(T);

impl<T: Shl<u8, Output = T>, const P: u8> Fixed<T, P> {
    pub fn new_int(val: T) -> Self {
        Self(val << P)
    }
}

impl<T: Add<Output = T>, const P: u8> Add for Fixed<T, P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T: Sub<Output = T>, const P: u8> Sub for Fixed<T, P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<T: Mul<Output = T> + Shr<Output = T> + From<u8>, const P: u8> Mul for Fixed<T, P> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) >> T::from(P))
    }
}

impl<T: Div<Output = T> + Shl<Output = T> + From<u8>, const P: u8> Div for Fixed<T, P> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 << T::from(P)) / rhs.0)
    }
}

macro_rules! assign_impl {
        ($op:ident) => {
            paste! {
                impl<T: $op<Output = T> + Clone + Shr<Output = T> + Shl<Output = T> + From<u8>, const P: u8> [<$op Assign>] for Fixed<T, P> {
                    fn [<$op:lower _assign>](&mut self, rhs: Self) {
                        *self = self.clone().[<$op:lower>](rhs);
                    }
                }
            }
        };
    }

assign_impl!(Add);
assign_impl!(Sub);
assign_impl!(Mul);
assign_impl!(Div);

impl<T: Add<Output = T> + Zero, const P: u8> Zero for Fixed<T, P> {
    fn zero() -> Self {
        Self(T::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: NumCast, const P: u8> From<f32> for Fixed<T, P> {
    fn from(value: f32) -> Self {
        Fixed(
            T::from(f32::round(
                value * f32::powi(2.0, <i32 as From<u8>>::from(P)),
            ))
            .unwrap(),
        )
    }
}

impl<T: ToPrimitive, const P: u8> From<Fixed<T, P>> for f32 {
    fn from(value: Fixed<T, P>) -> Self {
        value.0.to_f32().unwrap_or(f32::NAN) * f32::powi(2.0, -<i32 as From<u8>>::from(P))
    }
}

pub type Vec3 = nalgebra::Vector3<Fixed<i32, 14>>;
