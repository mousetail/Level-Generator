use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
mod generate_level;
mod physics;
mod rotate_camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .insert_resource(
        // )
        .add_plugin(physics::PlayerControllerPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(rotate_camera::PlayerPlugin)
        .add_startup_system(setup_level)
        .add_system(rotate_camera_system)
        .run();
}

#[derive(Component)]
struct RotateCamera(f32);

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // level
    generate_level::generate_level(&mut commands, &mut meshes, &mut materials, asset_server);
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(6.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(6.0 * 3., 8.0, 8.0 * 3.),
        ..default()
    });

    // commands
    //     .spawn()
    //     .insert_bundle(Camera3dBundle {
    //         transform: Transform::from_xyz(12.0, 15.0, 6.0)
    //             .looking_at(Vec3::new(6.0, 0.0, 6.0), Vec3::Y),
    //         ..Default::default()
    //     })
    //     .insert(RotateCamera(0.0));
}

fn rotate_camera_system(
    mut cameras: Query<(&mut RotateCamera, &mut Transform)>,
    players: Query<(&physics::PlayerController, &GlobalTransform), Without<RotateCamera>>,
) {
    let mut player_pos = Vec3::ZERO;

    for player in players.iter() {
        player_pos = player.1.translation();
        // println!("{:?}", player_pos);
    }

    for mut camera in cameras.iter_mut() {
        camera.0 .0 += 0.01;
        *camera.1 = camera.1.looking_at(player_pos, Vec3::Y)
    }
}
