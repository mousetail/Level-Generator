use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
mod cubemap;
mod generate_level;
mod normal_mapped_texture_loader;
mod physics;
mod rotate_camera;
mod util;
use bevy::render::render_resource::{AddressMode, SamplerDescriptor};
use bevy::render::texture::ImageSettings;

fn main() {
    App::new()
        .insert_resource(ImageSettings {
            default_sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..default()
            },
        })
        .add_plugins(DefaultPlugins)
        .init_asset_loader::<normal_mapped_texture_loader::NormalMappedImageTextureLoader>()
        .insert_resource(bevy::asset::AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        // .insert_resource(
        // )
        .add_plugin(physics::PlayerControllerPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(rotate_camera::PlayerPlugin)
        .add_plugin(cubemap::CubemapPlugin)
        .add_startup_system(setup_level)
        .add_system(rotate_camera_system)
        .add_startup_system(enable_hot_reloading)
        // .add_system(fix_normal_mapped_textures::set_image_color_mode)
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
    // commands.spawn_bundle(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 1500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(6.0, 8.0, 4.0),
    //     ..default()
    // });

    // commands.spawn_bundle(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 1500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(6.0 * 3., 8.0, 8.0 * 3.),
    //     ..default()
    // });

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            color: Color::rgb(1.0, 1.0, 0.9),
            // shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_8)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_6 * 2.),
            ..default()
        },
        ..default()
    });

    commands.insert_resource(AmbientLight {
        brightness: 0.025,
        color: Color::rgb(0.25, 0.25, 1.0),
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

fn enable_hot_reloading(asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
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
