use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct PlayerController;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(print_player_position);
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(Color::rgb(0.9, 0.3, 0.9).into());
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert_bundle(TransformBundle::from(
            Transform::from_xyz(6.0 * 3.0, 12.0, 6.0 * 3.0), //.looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ))
        .insert(Velocity {
            linvel: Vec3::ZERO,
            ..Default::default()
        })
        .insert(Collider::capsule_y(0.75, 0.25))
        // .insert(Restitution::coefficient(0.7))
        .insert(
            LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_Y,
        )
        .insert(ExternalImpulse {
            impulse: Vec3::ZERO,
            ..Default::default()
        })
        .insert(Friction {
            coefficient: 0.0,
            ..default()
        })
        .insert(Damping {
            linear_damping: 8.0,
            ..default()
        })
        .insert(GravityScale(2.0))
        // .insert(Sleeping::disabled())
        .with_children(|parent| {
            // let mut parent = commands;

            parent
                .spawn()
                .insert_bundle(Camera3dBundle {
                    transform: Transform::from_xyz(0., 0., 0.)
                        .looking_at(Vec3::new(0., 0., 6.), Vec3::Y),
                    projection: bevy::render::camera::Projection::Perspective(
                        PerspectiveProjection {
                            fov: 1.5,
                            near: 0.01,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                })
                .insert(crate::rotate_camera::FlyCam);
            parent
                .spawn()
                .insert_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(0., -0.5, 0.5),
                    ..default()
                })
                .insert(PlayerController);
        });
}

fn print_player_position() { //query: Query<(&Transform, &PlayerController)>) {
                             // for item in query.iter() {
                             //     println!("{:?}", item.0.translation);
                             // }
}
