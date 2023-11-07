use serde::{Deserialize, Serialize};

pub struct CompactBasis(pub [i8; 3]);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingFacing {
    Px,
    Nx,
    Py,
    Ny,
    Pz,
    Nz,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingRotation {
    N,
    W,
    S,
    E,
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
        let is_y = match self {
            BuildingFacing::Py | BuildingFacing::Ny => true,
            _ => false,
        };
        if is_y {
            transitions[((key + 4 - current) % 4) as usize]
        } else {
            transitions[key as usize]
        }
    }
}
