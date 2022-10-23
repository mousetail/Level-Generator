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
    decorator::decorate_level(&grid, commands, materials, asset_server);

    let floor_plane = meshes.add(Mesh::from(shape::Plane { size: 3. * 15.0 }));
    commands.spawn_bundle(PbrBundle {
        mesh: floor_plane,
        transform: Transform::from_xyz(6.0 * 3.0, -2.5, 6.0 * 3.0),
        material: materials.add(Color::rgb(0., 0., 0.).into()),
        ..default()
    });
}
