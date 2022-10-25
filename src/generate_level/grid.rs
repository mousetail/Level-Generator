use bevy::prelude::*;
use std::f32::consts;
pub(super) const LEVEL_SIZE: (usize, usize, usize) = (12, 12, 3);
pub(super) const LEVEL_SCALE: (f32, f32, f32) = (3., 3., 2.5);

pub struct LevelGrid(
    [[[GridTile; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0],
    pub [[u8; LEVEL_SIZE.1]; LEVEL_SIZE.0],
);
pub const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, -1), (0, 1)];

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

    pub fn can_access(self, direction: (isize, isize)) -> bool {
        // if self.is_bottom_stair_tile() && direction == (0, 1) {
        //     return true;
        // }
        // if self.is_top_stair_tile() && direction == (0, -1) {
        //     return true;
        // }
        match (self, direction) {
            (GridTile::Empty, _) => false,
            (GridTile::Floor, (0, 1) | (0, -1) | (1, 0) | (-1, 0)) => true,
            (GridTile::Floor, _) => false,
            (GridTile::StairsNorthBottom | GridTile::StairsSouthTop, (0, -1)) => true,
            (GridTile::StairsSouthBottom | GridTile::StairsNorthTop, (0, 1)) => true,
            (GridTile::StairsEastBottom | GridTile::StairsWestTop, (1, 0)) => true,
            (GridTile::StairsWestBottom | GridTile::StairsEastTop, (-1, 0)) => true,
            _ => false,
        }
    }
}

impl LevelGrid {
    pub fn new(max_height: [[u8; LEVEL_SIZE.1]; LEVEL_SIZE.0]) -> LevelGrid {
        LevelGrid(
            [[[GridTile::Empty; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0],
            max_height,
        )
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

    pub fn is_valid_height(&self, x: isize, y: isize, z: isize) -> bool {
        if x < 0
            || x >= LEVEL_SIZE.0 as isize
            || y < 0
            || y >= LEVEL_SIZE.1 as isize
            || z < 0
            || z >= LEVEL_SIZE.2 as isize
        {
            return false;
        }

        let max_height = self.1[x as usize][y as usize];
        return z <= max_height as isize && z + 1 >= max_height as isize;
    }

    pub fn get_max_height(&self, x: isize, y: isize) -> u8 {
        if x < 0 || x >= LEVEL_SIZE.0 as isize || y < 0 || y >= LEVEL_SIZE.1 as isize {
            return 0;
        }

        return self.1[x as usize][y as usize];
    }

    pub fn can_access(&self, p1: (isize, isize, isize), p2: (isize, isize, isize)) -> bool {
        let t1 = self.get(p1.0, p1.1, p1.2);
        let t2 = self.get(p2.0, p2.1, p2.2);
        if (p1.0, p1.1, p1.2) == (p2.0, p2.1, p1.2 + 1) {
            if t1.is_bottom_stair_tile() && t2.is_top_stair_tile() {
                return true;
            }
        }
        if (p1.0, p1.1, p1.2) == (p2.0, p2.1, p1.2 - 1) {
            if t2.is_bottom_stair_tile() && t1.is_top_stair_tile() {
                return true;
            }
        }
        if (p1.2 != p2.2) {
            return false;
        }

        return t1.can_access((p2.0 - p1.0, p2.1 - p1.1))
            && t2.can_access((p1.0 - p2.0, p1.1 - p2.1));
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
