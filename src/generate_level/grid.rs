use bevy::prelude::*;
use std::f32::consts;
pub(super) const LEVEL_SIZE: (usize, usize, usize) = (12, 12, 3);
pub(super) const LEVEL_SCALE: (f32, f32, f32) = (3., 3., 2.5);

pub struct LevelGrid([[[GridTile; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0]);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GridTile {
    Empty,
    Floor,
    StairsEastTop,
    StairsEastBottom,
    StairsNorthTop,
    StairsNorthBottom,
    StairsWestTop,
    StairsWestBottom,
    StairsSouthTop,
    StairsSouthBottom,
    OutOfBounds,
}

impl GridTile {
    pub fn is_stair_tile(self) -> bool {
        match self {
            GridTile::StairsEastTop
            | GridTile::StairsEastBottom
            | GridTile::StairsNorthTop
            | GridTile::StairsNorthBottom
            | GridTile::StairsWestTop
            | GridTile::StairsWestBottom
            | GridTile::StairsSouthTop
            | GridTile::StairsSouthBottom => true,
            _ => false,
        }
    }

    pub fn is_top_stair_tile(self) -> bool {
        match self {
            GridTile::StairsEastTop
            | GridTile::StairsNorthTop
            | GridTile::StairsWestTop
            | GridTile::StairsSouthTop => true,
            _ => false,
        }
    }

    pub fn is_bottom_stair_tile(self) -> bool {
        match self {
            GridTile::StairsEastBottom
            | GridTile::StairsNorthBottom
            | GridTile::StairsWestBottom
            | GridTile::StairsSouthBottom => true,
            _ => false,
        }
    }

    pub fn get_opposite_stair_tile(self) -> Option<GridTile> {
        match self {
            GridTile::StairsEastBottom => Some(GridTile::StairsEastTop),
            GridTile::StairsEastTop => Some(GridTile::StairsEastBottom),
            GridTile::StairsNorthBottom => Some(GridTile::StairsNorthTop),
            GridTile::StairsNorthTop => Some(GridTile::StairsNorthBottom),
            GridTile::StairsSouthBottom => Some(GridTile::StairsSouthTop),
            GridTile::StairsSouthTop => Some(GridTile::StairsSouthBottom),
            GridTile::StairsWestBottom => Some(GridTile::StairsWestTop),
            GridTile::StairsWestTop => Some(GridTile::StairsWestBottom),
            _ => None,
        }
    }

    pub fn get_angle(self) -> Quat {
        match self {
            GridTile::StairsEastBottom | GridTile::StairsEastTop => {
                Quat::from_rotation_y(consts::PI * 0.)
            }
            GridTile::StairsNorthTop | GridTile::StairsNorthBottom => {
                Quat::from_rotation_y(1. * consts::FRAC_PI_2)
            }
            GridTile::StairsWestTop | GridTile::StairsWestBottom => {
                Quat::from_rotation_y(consts::PI)
            }
            GridTile::StairsSouthTop | GridTile::StairsSouthBottom => {
                Quat::from_rotation_y(3. * consts::FRAC_PI_2)
            }
            _ => Quat::IDENTITY,
        }
    }
}

impl LevelGrid {
    pub fn new() -> LevelGrid {
        LevelGrid([[[GridTile::Empty; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0])
    }

    pub fn get(&self, x: isize, y: isize, z: isize) -> GridTile {
        if x < 0
            || x >= LEVEL_SIZE.0 as isize
            || y < 0
            || y >= LEVEL_SIZE.1 as isize
            || z < 0
            || z >= LEVEL_SIZE.2 as isize
        {
            return GridTile::OutOfBounds;
        }
        return self.0[x as usize][y as usize][z as usize];
    }

    pub fn set(&mut self, x: isize, y: isize, z: isize, val: GridTile) {
        self.0[x as usize][y as usize][z as usize] = val;
    }
}

impl IntoIterator for &LevelGrid {
    type Item = (isize, isize, isize);
    type IntoIter = GridIterator;

    fn into_iter(self) -> GridIterator {
        GridIterator(0, 0, -1)
    }
}

pub struct GridIterator(isize, isize, isize);

impl Iterator for GridIterator {
    type Item = (isize, isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        match (
            self.0 - LEVEL_SIZE.0 as isize + 1,
            self.1 - LEVEL_SIZE.1 as isize + 1,
            self.2 - LEVEL_SIZE.2 as isize + 1,
        ) {
            (0, 0, 0) => None,
            (_, 0, 0) => {
                self.0 += 1;
                self.1 = 0;
                self.2 = 0;
                Some((self.0, self.1, self.2))
            }
            (_, _, 0) => {
                self.1 += 1;
                self.2 = 0;
                Some((self.0, self.1, self.2))
            }
            (_, _, _) => {
                self.2 += 1;
                Some((self.0, self.1, self.2))
            }
        }
    }
}
