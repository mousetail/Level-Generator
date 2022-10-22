use super::grid::{GridTile, LevelGrid, DIRECTIONS, LEVEL_SCALE, LEVEL_SIZE};
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
    grid: &LevelGrid,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let floor_mesh: Handle<Mesh> = asset_server.load("Floor.glb#Mesh0/Primitive0");
    let stairs: Handle<Mesh> = asset_server.load("Stairs.glb#Mesh0/Primitive0");
    let railing_model: Handle<Mesh> = asset_server.load("Railing.glb#Mesh0/Primitive0");
    let pillar_short_model: Handle<Mesh> = asset_server.load("PillarShort.glb#Mesh0/Primitive0");
    let pillar_tall_model: Handle<Mesh> = asset_server.load("PillarTall.glb#Mesh0/Primitive0");
    let wall_model: Handle<Mesh> = asset_server.load("Wall.glb#Mesh0/Primitive0");
    let roof_model: [Handle<Mesh>; 3] = [
        asset_server.load("Roof.glb#Mesh0/Primitive0"),
        asset_server.load("Roof.glb#Mesh0/Primitive1"),
        asset_server.load("Roof.glb#Mesh0/Primitive2"),
    ];

    let floor_texture: Handle<Image> =
        asset_server.load("textures/Marble_TilesDiagonal2_512_albedo.png");
    let floor_material = materials.add(StandardMaterial {
        base_color_texture: Some(floor_texture.clone()),
        ..default()
    });

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
    let material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());

    for (x, y, z) in grid {
        if grid.get(x, y, z) != GridTile::Empty && !grid.get(x, y, z).is_top_stair_tile() {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: if grid.get(x, y, z).is_bottom_stair_tile() {
                        stairs.clone()
                    } else {
                        floor_mesh.clone()
                    },
                    material: if grid.get(x, y, z).is_stair_tile() {
                        material.clone()
                    } else {
                        floor_material.clone()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * LEVEL_SCALE.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1,
                    )
                    .with_rotation(grid.get(x, y, z).get_angle()),
                    // .with_rotation(Quat::from_rotation_x(
                    //     ((rng.gen::<f32>() - 0.5) * 20.0_f32).to_radians(),
                    // )),
                    ..default()
                })
                .insert(if grid.get(x, y, z).is_bottom_stair_tile() {
                    stair_collider.clone()
                } else {
                    Collider::cuboid(1.5, 0.1, 1.5)
                });
        }
    }

    for (x, y, z) in grid {
        if grid.get(x, y, z) != GridTile::Floor {
            continue;
        }

        for direction in DIRECTIONS {
            if !grid
                .get(x + direction.0, y + direction.1, z)
                .can_access((-direction.0, -direction.1))
            {
                let angle = Quat::from_rotation_y(
                    (direction.0 as f32).atan2(direction.1 as f32) + consts::PI,
                );

                if grid.1[x as usize][y as usize] != 1 {
                    spawn_railing(
                        commands,
                        Vec3::new(x as f32 * 3., z as f32 * 2.5, y as f32 * 3.)
                            + angle * Vec3::new(-1.5, 0., -1.5),
                        angle,
                        railing_model.clone(),
                        material.clone(),
                    )
                }
            }
        }
    }

    for x in 0..LEVEL_SIZE.0 {
        for y in 0..LEVEL_SIZE.1 {
            if grid.1[x][y] == 1 {
                for model in &roof_model {
                    commands.spawn_bundle(PbrBundle {
                        mesh: model.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * LEVEL_SCALE.0,
                            2. * LEVEL_SCALE.2,
                            y as f32 * LEVEL_SCALE.1,
                        ),
                        ..default()
                    });
                }
            }
        }
    }

    let is_covered = |x, y, z| {
        grid.get_max_height(x, y) == 1// grid.get(x, y, z) == GridTile::Empty
        && matches!(grid.get(x, y, z), GridTile::Empty | GridTile::OutOfBounds)
    };

    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in 0..2 {
                if is_covered(x - 1, y, z) != is_covered(x, y, z) {
                    // println!(
                    //     "is covered {} {} {} = {} / {}",
                    //     x,
                    //     y,
                    //     z,
                    //     is_covered(x, y, z),
                    //     is_covered(x - 1, y, z)
                    // );
                    commands.spawn_bundle(PbrBundle {
                        mesh: wall_model.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * LEVEL_SCALE.0 - LEVEL_SCALE.0 / 2.0,
                            z as f32 * LEVEL_SCALE.2,
                            y as f32 * LEVEL_SCALE.1 - LEVEL_SCALE.1 / 2.0,
                        )
                        .with_rotation(Quat::from_rotation_y(-consts::FRAC_PI_2)),
                        ..default()
                    });
                }

                if is_covered(x, y - 1, z) != is_covered(x, y, z) {
                    commands.spawn_bundle(PbrBundle {
                        mesh: wall_model.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * LEVEL_SCALE.0 - LEVEL_SCALE.0 / 2.0,
                            z as f32 * LEVEL_SCALE.2,
                            y as f32 * LEVEL_SCALE.1 - LEVEL_SCALE.1 / 2.0,
                        ),
                        //.with_rotation(Quat::from_rotation_y(-consts::FRAC_PI_2)),
                        ..default()
                    });
                }
            }
        }
    }

    let is_any_plane = |x: isize, y: isize, z: isize| -> u8 {
        if z >= LEVEL_SIZE.2 as isize {
            return 0;
        }
        return (grid.get(x - 1, y - 1, z) == GridTile::Floor || is_covered(x - 1, y - 1, z)) as u8
            + (grid.get(x - 1, y, z) == GridTile::Floor || is_covered(x - 1, y, z)) as u8
            + (grid.get(x, y - 1, z) == GridTile::Floor || is_covered(x, y - 1, z)) as u8
            + (grid.get(x, y, z) == GridTile::Floor || is_covered(x, y, z)) as u8;
    };

    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in 0..LEVEL_SIZE.2 as isize {
                let edges = is_any_plane(x, y, z);
                if edges > 0 && edges < 4 {
                    commands.spawn_bundle(PbrBundle {
                        mesh: if is_any_plane(x, y, z + 1) > 0 {
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
