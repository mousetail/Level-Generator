use bevy::prelude::*;

mod decorator;
mod generator;
mod grid;

pub fn generate_level(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let grid = generator::generate_level_grid();
    decorator::decorate_level(&grid, commands, meshes, materials, asset_server);
}
