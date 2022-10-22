use super::{GridTile, LevelGrid, LEVEL_SCALE};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts;

fn spawn_railing(
    commands: &mut Commands,
    position: Vec3,
    angle: Quat,
    model: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: model.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(position.x, position.y, position.z).with_rotation(angle),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_xyz(1.5, 0.25, 0.),
                    ..default()
                })
                .insert(Collider::cuboid(1.5, 0.25, 0.1));
        });
}

pub(super) fn decorate_level(
    grid: Box<LevelGrid>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let stairs: Handle<Mesh> = asset_server.load("Stairs.glb#Mesh0/Primitive0");
    let railing_model: Handle<Mesh> = asset_server.load("Railing.glb#Mesh0/Primitive0");
    let pillar_short_model: Handle<Mesh> = asset_server.load("PillarShort.glb#Mesh0/Primitive0");
    let pillar_tall_model: Handle<Mesh> = asset_server.load("PillarTall.glb#Mesh0/Primitive0");

    let stair_collider = Collider::convex_hull(&[
        Vec3::new(1.5, 0.05, -1.5),
        Vec3::new(1.5, 0.05, 1.5),
        Vec3::new(-1.5, 2.6, -1.5),
        Vec3::new(-1.5, 2.6, 1.5),
        Vec3::new(-1.5, 0.05, -1.5),
        Vec3::new(-1.5, 0.05, 1.5),
    ])
    .unwrap();

    let mesh = meshes.add(Mesh::from(shape::Box {
        min_x: -1.5,
        min_y: -0.1,
        min_z: -1.5,
        max_x: 1.5,
        max_y: -0.0,
        max_z: 1.5,
    }));
    let material = materials.add(Color::rgb(0.4, 0.7, 0.3).into());
    let stair_material = materials.add(Color::rgb(0.4, 0.7, 0.8).into());
    let stair_top_material = materials.add(Color::rgb(0.8, 0.4, 0.3).into());

    for x in 0..grid.len() {
        for y in 0..grid[x].len() {
            for z in 0..grid[x][y].len() {
                if grid[x][y][z] != GridTile::Empty && !grid[x][y][z].is_top_stair_tile() {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: if grid[x][y][z].is_bottom_stair_tile() {
                                stairs.clone()
                            } else {
                                mesh.clone()
                            },
                            material: if grid[x][y][z].is_top_stair_tile() {
                                stair_material.clone()
                            } else if grid[x][y][z].is_bottom_stair_tile() {
                                stair_top_material.clone()
                            } else {
                                material.clone()
                            },
                            transform: Transform::from_xyz(
                                x as f32 * LEVEL_SCALE.0,
                                z as f32 * LEVEL_SCALE.2,
                                y as f32 * LEVEL_SCALE.1,
                            )
                            .with_rotation(grid[x][y][z].get_angle()),
                            // .with_rotation(Quat::from_rotation_x(
                            //     ((rng.gen::<f32>() - 0.5) * 20.0_f32).to_radians(),
                            // )),
                            ..default()
                        })
                        .insert(if grid[x][y][z].is_bottom_stair_tile() {
                            stair_collider.clone()
                        } else {
                            Collider::cuboid(1.5, 0.1, 1.5)
                        });
                }
            }
        }
    }

    for x in 0..grid.len() {
        for y in 0..grid[x].len() {
            for z in 0..grid[x][y].len() {
                if grid[x][y][z] != GridTile::Floor {
                    continue;
                }

                if x == grid.len() - 1 || grid[x + 1][y][z] == GridTile::Empty {
                    spawn_railing(
                        commands,
                        Vec3::new(x as f32 * 3. + 1.5, z as f32 * 2.5, y as f32 * 3. + 1.5),
                        Quat::from_rotation_y(consts::FRAC_PI_2),
                        railing_model.clone(),
                        material.clone(),
                    )
                }
                if x == 0 || grid[x - 1][y][z] == GridTile::Empty {
                    spawn_railing(
                        commands,
                        Vec3::new(x as f32 * 3. - 1.5, z as f32 * 2.5, y as f32 * 3. + 1.5),
                        Quat::from_rotation_y(consts::FRAC_PI_2),
                        railing_model.clone(),
                        material.clone(),
                    )
                }
                if y == grid[x].len() - 1 || grid[x][y + 1][z] == GridTile::Empty {
                    spawn_railing(
                        commands,
                        Vec3::new(x as f32 * 3. - 1.5, z as f32 * 2.5, y as f32 * 3. + 1.5),
                        Quat::from_rotation_y(0.),
                        railing_model.clone(),
                        material.clone(),
                    )
                }
                if y == 0 || grid[x][y - 1][z] == GridTile::Empty {
                    spawn_railing(
                        commands,
                        Vec3::new(x as f32 * 3. - 1.5, z as f32 * 2.5, y as f32 * 3. - 1.5),
                        Quat::from_rotation_y(0.),
                        railing_model.clone(),
                        material.clone(),
                    )
                }
            }
        }
    }

    let is_any_plane = |x: usize, y: usize, z: usize| -> bool {
        if z >= grid[0][0].len() {
            return false;
        }
        return (x > 0 && y > 0 && grid[x - 1][y - 1][z] == GridTile::Floor)
            || (x > 0 && y < grid[x - 1].len() && grid[x - 1][y][z] == GridTile::Floor)
            || (y > 0 && x < grid.len() && grid[x][y - 1][z] == GridTile::Floor)
            || (x < grid.len() && y < grid[x].len() && grid[x][y][z] == GridTile::Floor);
    };

    for x in 0..grid.len() + 1 {
        for y in 0..grid.len() + 1 {
            for z in 0..grid.len() {
                if is_any_plane(x, y, z) {
                    commands.spawn_bundle(PbrBundle {
                        mesh: if is_any_plane(x, y, z + 1) {
                            pillar_tall_model.clone()
                        } else {
                            pillar_short_model.clone()
                        },
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * 3. - 1.5,
                            z as f32 * 2.5,
                            y as f32 * 3. - 1.5,
                        ),
                        ..default()
                    });
                }
            }
        }
    }
}
