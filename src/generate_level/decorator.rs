use super::grid::{GridTile, LevelGrid, DIRECTIONS, LEVEL_SCALE, LEVEL_SIZE};
use crate::util::is_all_same;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts;

struct SharedMaterials {
    floor: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    generic: Handle<StandardMaterial>,
    roof: Handle<StandardMaterial>,
}

impl SharedMaterials {
    fn new(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) -> SharedMaterials {
        SharedMaterials {
            floor: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Marble_TilesDiagonal2_512_albedo.png"),
                ),
                metallic_roughness_texture: Some(
                    asset_server.load("textures/Marble_tilesDiagonal2_512_roughness.png"),
                ),
                ..default()
            }),
            wall: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Plaster_Plain_512_albedo.png"),
                ),
                normal_map_texture: Some(asset_server.load("textures/Plaster_Plain_1k_normal.tga")),
                ..default()
            }),
            generic: materials.add(Color::rgb(1., 1., 1.).into()),
            roof: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Roofing_SquareOld2_1K_albedo.png"),
                ),
                normal_map_texture: Some(
                    asset_server.load("textures/Roofing_SquareOld2_1K_normal.png"),
                ),
                ..default()
            }),
        }
    }
}

fn spawn_railing(
    commands: &mut Commands,
    position: Vec3,
    angle: Quat,
    wall_type: WallType,
    model: [&Handle<Mesh>; 3],
    material: [&Handle<StandardMaterial>; 3],
) {
    if wall_type == WallType::None {
        return;
    }

    let index = match wall_type {
        WallType::Short => 0,
        WallType::Tall => 1,
        WallType::StairLeft | WallType::StairRight => 2,
        _ => panic!("Invalid wall type"),
    };

    let wall_height = match wall_type {
        WallType::Short => 0.25,
        WallType::Tall => 1.25,
        _ => 0.5,
    };

    commands
        .spawn_bundle(PbrBundle {
            mesh: model[index].clone(),
            material: material[index].clone(),
            transform: if wall_type == WallType::StairLeft
                || (wall_type == WallType::Tall && position.x as isize % 2 == 0)
            {
                Transform::from_translation(position + angle * Vec3::new(3., 0., 0.))
                    .with_rotation(Quat::from_rotation_y(consts::PI) * angle)
            } else {
                Transform::from_xyz(position.x, position.y, position.z).with_rotation(angle)
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TransformBundle {
                    local: Transform::from_xyz(1.5, wall_height, 0.),
                    ..default()
                })
                .insert(Collider::cuboid(1.5, wall_height, 0.1));
        });
}

fn is_walkable(grid: &LevelGrid, x: isize, y: isize, z: isize) -> bool {
    match grid.get(x, y, z) {
        GridTile::Empty | GridTile::OutOfBounds => false,
        _ => true,
    }
}

fn is_above_walkable(grid: &LevelGrid, x: isize, y: isize, z: isize) -> bool {
    for z2 in 0..=z {
        if is_walkable(grid, x, y, z2) {
            return true;
        }
    }
    return false;
}

fn is_indoor(grid: &LevelGrid, x: isize, y: isize) -> bool {
    grid.get_max_height(x, y) == 1
}

fn is_unreachable(grid: &LevelGrid, x: isize, y: isize, z: isize) -> bool {
    is_indoor(grid, x, y) && !is_above_walkable(grid, x, y, z)
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum WallType {
    None,
    Short,
    StairLeft,
    StairRight,
    Tall,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum PillarType {
    None,
    Short,
    Tall,
}

fn should_build_wall(
    grid: &LevelGrid,
    p1: (isize, isize, isize),
    p2: (isize, isize, isize),
) -> WallType {
    // Case 1: Edge of a walkable area and the void.
    if !is_above_walkable(grid, p1.0, p1.1, p1.2)
        && !is_above_walkable(grid, p2.0, p2.1, p2.2)
        && (p2.2..LEVEL_SIZE.2 as isize)
            .any(|z| is_walkable(grid, p1.0, p1.1, z) != is_above_walkable(grid, p2.0, p2.1, z))
    {
        return WallType::Tall;
    }

    if is_above_walkable(grid, p1.0, p1.1, p1.2) ^ is_above_walkable(grid, p2.0, p2.1, p2.2)
        && !grid.can_access(p1, p2)
        && is_above_walkable(grid, p1.0, p1.1, LEVEL_SIZE.2 as isize - 1)
        && is_above_walkable(grid, p2.0, p2.1, LEVEL_SIZE.2 as isize - 1)
    {
        return WallType::Tall;
    }

    if is_unreachable(grid, p1.0, p1.1, p1.2) != is_unreachable(grid, p2.0, p2.1, p2.2) && p1.2 <= 1
    {
        return WallType::Tall;
    } else if !(grid
        .get(p1.0, p1.1, p1.2)
        .can_access((p2.0 - p1.0, p2.1 - p1.1))
        && grid
            .get(p2.0, p2.1, p2.2)
            .can_access((p1.0 - p2.0, p1.1 - p2.1)))
        && (grid.get(p1.0, p1.1, p1.2).is_bottom_stair_tile()
            || grid.get(p2.0, p2.1, p2.2).is_bottom_stair_tile()
            || grid.get(p1.0, p1.1, p1.2) == GridTile::Floor
            || grid.get(p2.0, p2.1, p2.2) == GridTile::Floor)
    {
        if (grid.get(p1.0, p1.1, p1.2).is_bottom_stair_tile()
            && !grid
                .get(p1.0, p1.1, p1.2 + 1)
                .can_access((p2.0 - p1.0, p2.1 - p1.1)))
            || (grid.get(p2.0, p2.1, p2.2).is_bottom_stair_tile()
                && !grid
                    .get(p2.0, p2.1, p2.2 + 1)
                    .can_access((p1.0 - p2.0, p1.1 - p2.1)))
        {
            return match (grid.get(p1.0, p1.1, p2.2), grid.get(p2.0, p2.1, p2.2)) {
                (GridTile::StairsNorthBottom, _)
                | (_, GridTile::StairsNorthBottom)
                | (GridTile::StairsWestBottom, _)
                | (_, GridTile::StairsWestBottom) => WallType::StairRight,
                _ => WallType::StairLeft,
            };
        }
        return WallType::Short;
    }

    WallType::None
}

fn should_build_pillar(grid: &LevelGrid, pos: (isize, isize, isize)) -> PillarType {
    let corners = [
        (pos.0, pos.1, pos.2),
        (pos.0 - 1, pos.1, pos.2),
        (pos.0, pos.1 - 1, pos.2),
        (pos.0 - 1, pos.1 - 1, pos.2),
    ];

    if pos.2 < LEVEL_SIZE.2 as isize - 1
        && should_build_pillar(grid, (pos.0, pos.1, pos.2 + 1)) != PillarType::None
    {
        return PillarType::Tall;
    }

    if !is_all_same(corners.iter().map(|i| is_indoor(grid, i.0, i.1))) {
        return PillarType::Tall;
    }

    if corners
        .iter()
        .all(|i| !is_above_walkable(grid, i.0, i.1, i.2))
    {
        return PillarType::None;
    }

    if corners.iter().any(|i| is_indoor(grid, i.0, i.1)) {
        return PillarType::Tall;
    }

    if corners.iter().any(|i| is_walkable(grid, i.0, i.1, i.2 + 1)) {
        return PillarType::Tall;
    }

    if corners.iter().any(|i| is_walkable(grid, i.0, i.1, i.2))
        && !corners
            .iter()
            .all(|i| grid.get(i.0, i.1, i.2) == GridTile::Floor)
    {
        return PillarType::Short;
    }

    return PillarType::None;
}

fn add_floors(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &SharedMaterials,
    asset_server: &Res<AssetServer>,
) {
    let floor_mesh: Handle<Mesh> = asset_server.load("Floor.glb#Mesh0/Primitive0");
    let stairs_mesh: Handle<Mesh> = asset_server.load("Stairs.glb#Mesh0/Primitive0");
    let stairs_bottom_mesh: Handle<Mesh> = asset_server.load("UnderStairs.glb#Mesh0/Primitive0");

    let stair_collider = Collider::convex_hull(&[
        Vec3::new(1.5, 0.05, -1.5),
        Vec3::new(1.5, 0.05, 1.5),
        Vec3::new(-1.5, 2.6, -1.5),
        Vec3::new(-1.5, 2.6, 1.5),
        Vec3::new(-1.5, 0.05, -1.5),
        Vec3::new(-1.5, 0.05, 1.5),
    ])
    .unwrap();

    for (x, y, z) in grid {
        if grid.get(x, y, z) != GridTile::Empty && !grid.get(x, y, z).is_top_stair_tile() {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: if grid.get(x, y, z).is_bottom_stair_tile() {
                        stairs_mesh.clone()
                    } else {
                        floor_mesh.clone()
                    },
                    material: if grid.get(x, y, z).is_stair_tile() {
                        materials.generic.clone()
                    } else {
                        materials.floor.clone()
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

            if grid.get(x, y, z).is_stair_tile() {
                commands.spawn_bundle(PbrBundle {
                    mesh: stairs_bottom_mesh.clone(),
                    material: materials.wall.clone(),
                    transform: Transform::from_xyz(
                        x as f32 * LEVEL_SCALE.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1,
                    )
                    .with_rotation(grid.get(x, y, z).get_angle()),
                    ..default()
                });
            }
        }
    }
}

fn add_roofs(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &SharedMaterials,
    asset_server: &Res<AssetServer>,
) {
    let roof_model: [Handle<Mesh>; 3] = [
        asset_server.load("Roof.glb#Mesh0/Primitive0"),
        asset_server.load("Roof.glb#Mesh0/Primitive1"),
        asset_server.load("Roof.glb#Mesh0/Primitive2"),
    ];
    let materials = [
        materials.floor.clone(),
        materials.wall.clone(),
        materials.roof.clone(),
    ];

    for x in 0..LEVEL_SIZE.0 {
        for y in 0..LEVEL_SIZE.1 {
            if grid.1[x][y] == 1 && grid.get(x as isize, y as isize, 2) == GridTile::Empty {
                for (model, material) in roof_model.iter().zip(materials.iter()) {
                    commands.spawn_bundle(PbrBundle {
                        mesh: model.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * LEVEL_SCALE.0,
                            2. * LEVEL_SCALE.2,
                            y as f32 * LEVEL_SCALE.1,
                        )
                        .with_rotation(Quat::from_rotation_y(
                            if y % 2 == 0 { consts::PI } else { 0. },
                        )),
                        ..default()
                    });
                }
            }
        }
    }
}

fn add_walls(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &SharedMaterials,
    asset_server: &Res<AssetServer>,
) {
    let railing_model: Handle<Mesh> = asset_server.load("Railing.glb#Mesh0/Primitive0");
    let wall_model: Handle<Mesh> = asset_server.load("Wall.glb#Mesh0/Primitive0");
    let stair_railing_model: Handle<Mesh> = asset_server.load("StairRailing.glb#Mesh0/Primitive0");

    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in -1..LEVEL_SIZE.2 as isize {
                spawn_railing(
                    commands,
                    Vec3::new(
                        x as f32 * LEVEL_SCALE.0 - LEVEL_SCALE.0 / 2.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1 - LEVEL_SCALE.1 / 2.0,
                    ),
                    Quat::from_rotation_y(-consts::FRAC_PI_2),
                    should_build_wall(grid, (x - 1, y, z), (x, y, z)),
                    [&railing_model, &wall_model, &stair_railing_model],
                    [&materials.generic, &materials.wall, &materials.generic],
                );

                spawn_railing(
                    commands,
                    Vec3::new(
                        x as f32 * LEVEL_SCALE.0 - LEVEL_SCALE.0 / 2.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1 - LEVEL_SCALE.1 / 2.0,
                    ),
                    Quat::default(),
                    should_build_wall(grid, (x, y - 1, z), (x, y, z)),
                    [&railing_model, &wall_model, &stair_railing_model],
                    [&materials.generic, &materials.wall, &materials.generic],
                );
            }
        }
    }
}

fn add_pillars(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &SharedMaterials,
    asset_server: &Res<AssetServer>,
) {
    let pillar_short_model: Handle<Mesh> = asset_server.load("PillarShort.glb#Mesh0/Primitive0");
    let pillar_tall_model: Handle<Mesh> = asset_server.load("PillarTall.glb#Mesh0/Primitive0");

    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in -1..LEVEL_SIZE.2 as isize {
                let edges = should_build_pillar(grid, (x, y, z));
                if edges != PillarType::None {
                    commands.spawn_bundle(PbrBundle {
                        mesh: if edges == PillarType::Tall {
                            pillar_tall_model.clone()
                        } else {
                            pillar_short_model.clone()
                        },
                        material: materials.generic.clone(),
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

pub(super) fn decorate_level(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let shared_materials = SharedMaterials::new(materials, &asset_server);

    add_floors(grid, commands, &shared_materials, &asset_server);
    add_roofs(grid, commands, &shared_materials, &asset_server);
    add_walls(grid, commands, &shared_materials, &asset_server);
    add_pillars(grid, commands, &shared_materials, &asset_server);
}
