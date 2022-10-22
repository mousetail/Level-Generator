use bevy::prelude::*;

use std::f32::consts;
mod decorator;
mod generator;

const LEVEL_SIZE: (usize, usize, usize) = (12, 12, 3);
const LEVEL_SCALE: (f32, f32, f32) = (3., 3., 2.5);

type LevelGrid = [[[GridTile; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0];

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum GridTile {
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
    fn is_stair_tile(self) -> bool {
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

    fn is_top_stair_tile(self) -> bool {
        match self {
            GridTile::StairsEastTop
            | GridTile::StairsNorthTop
            | GridTile::StairsWestTop
            | GridTile::StairsSouthTop => true,
            _ => false,
        }
    }

    fn is_bottom_stair_tile(self) -> bool {
        match self {
            GridTile::StairsEastBottom
            | GridTile::StairsNorthBottom
            | GridTile::StairsWestBottom
            | GridTile::StairsSouthBottom => true,
            _ => false,
        }
    }

    fn get_opposite_stair_tile(self) -> Option<GridTile> {
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

    fn get_angle(self) -> Quat {
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

fn get_grid_at(grid: &LevelGrid, x: isize, y: isize, z: isize) -> GridTile {
    if x < 0
        || x >= LEVEL_SIZE.0 as isize
        || y < 0
        || y >= LEVEL_SIZE.1 as isize
        || z < 0
        || z >= LEVEL_SIZE.2 as isize
    {
        return GridTile::OutOfBounds;
    }
    return grid[x as usize][y as usize][z as usize];
}

pub fn generate_level(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let grid = generator::generate_level_grid();
    decorator::decorate_level(grid, commands, meshes, materials, asset_server);
}
