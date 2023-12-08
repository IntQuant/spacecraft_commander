use std::mem;

use serde::{Deserialize, Serialize};

pub struct CompactBasis(pub [i8; 3]);

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum BuildingFacing {
    #[default]
    Px,
    Nx,
    Py,
    Ny,
    Pz,
    Nz,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum BuildingRotation {
    #[default]
    N,
    E,
    S,
    W,
}
impl BuildingRotation {
    pub fn turn(self, by: i32) -> Self {
        let index = self as i32;
        let res_index = (index + by).rem_euclid(4);
        Self::from_index(res_index as u8)
    }

    fn from_index(res_index: u8) -> BuildingRotation {
        match res_index {
            0 => Self::N,
            1 => Self::E,
            2 => Self::S,
            3 => Self::W,
            _ => panic!("Unexpected index for BuildingRotation"),
        }
    }
}

impl BuildingFacing {
    pub fn to_basis(&self) -> CompactBasis {
        match self {
            BuildingFacing::Px => CompactBasis([1, 2, 3]),
            BuildingFacing::Py => CompactBasis([2, 1, 3]),
            BuildingFacing::Pz => CompactBasis([3, 2, -1]),
            BuildingFacing::Nx => CompactBasis([-1, 2, -3]),
            BuildingFacing::Ny => CompactBasis([-2, 1, 3]),
            BuildingFacing::Nz => CompactBasis([-3, 2, 1]),
        }
    }
    fn transitions(&self) -> [BuildingFacing; 4] {
        match self {
            BuildingFacing::Px => [Self::Pz, Self::Py, Self::Nz, Self::Ny],
            BuildingFacing::Nx => [Self::Nz, Self::Py, Self::Pz, Self::Ny],
            BuildingFacing::Py | BuildingFacing::Ny => [Self::Px, Self::Pz, Self::Nx, Self::Nz],
            BuildingFacing::Pz => [Self::Nx, Self::Py, Self::Px, Self::Ny],
            BuildingFacing::Nz => [Self::Px, Self::Py, Self::Nx, Self::Ny],
        }
    }
    pub fn turn(&self, key: u8, current: u8) -> Self {
        let transitions = self.transitions();
        let is_y = matches!(self, BuildingFacing::Py | BuildingFacing::Ny);
        if is_y {
            transitions[((key + 4 - current) % 4) as usize]
        } else {
            transitions[key as usize]
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BuildingOrientation {
    pub facing: BuildingFacing,
    pub rotation: BuildingRotation,
}

impl BuildingOrientation {
    pub fn new(facing: BuildingFacing, rotation: BuildingRotation) -> Self {
        Self { facing, rotation }
    }
    pub fn to_basis(&self) -> CompactBasis {
        self.facing.to_basis().rotate_by(self.rotation)
    }
}

impl CompactBasis {
    pub fn for_buildings(self) -> CompactBasis {
        let mut ret = CompactBasis(self.0);
        let split = ret.0.split_first_mut().unwrap();
        mem::swap(split.0, &mut split.1[0]);
        ret
    }

    pub fn rotate_by(self, current_rotation: BuildingRotation) -> CompactBasis {
        let rot_basis = match current_rotation {
            BuildingRotation::N => CompactBasis([1, 2, 3]),
            BuildingRotation::W => CompactBasis([1, -3, 2]),
            BuildingRotation::S => CompactBasis([1, -2, -3]),
            BuildingRotation::E => CompactBasis([1, 3, -2]),
        };
        let mut new_basis_data = [0; 3];
        for i in 0..3 {
            let basis_index_raw = rot_basis.0[i];
            let (basis_index, mul) = if basis_index_raw < 0 {
                (-basis_index_raw - 1, -1)
            } else {
                (basis_index_raw - 1, 1)
            };
            new_basis_data[i] = self.0[basis_index as usize] * mul;
        }
        CompactBasis(new_basis_data)
    }
}
