use super::grid::{GridTile, LevelGrid, DIRECTIONS, LEVEL_SCALE, LEVEL_SIZE};
use super::shared_models::{SharedMaterials, SharedModels};
use crate::util::is_all_same;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use std::f32::consts;

const ROOF_HEIGHT: isize = 3;

fn spawn_railing(
    commands: &mut Commands,
    position: Vec3,
    angle: Quat,
    wall_type: WallType,
    models: &SharedModels,
) {
    if wall_type == WallType::None {
        return;
    }

    let model = match wall_type {
        WallType::Short => &models.railing,
        WallType::Tall => &models.wall,
        WallType::StairLeft | WallType::StairRight => &models.stair_railing,
        WallType::Window => &models.wall_window,
        _ => panic!("Invalid wall type"),
    };

    // let wall_height = match wall_type {
    //     WallType::Short => 0.25,
    //     WallType::Tall => 1.25,
    //     _ => 0.5,
    // };

    model.build(
        if wall_type == WallType::StairLeft
            || (wall_type == WallType::Tall && position.x as isize % 2 == 0)
        {
            Transform::from_translation(position + angle * Vec3::new(3., 0., 0.))
                .with_rotation(Quat::from_rotation_y(consts::PI) * angle)
        } else {
            Transform::from_xyz(position.x, position.y, position.z).with_rotation(angle)
        },
        commands.spawn(),
    );

    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: model[index].clone(),
    //         material: material[index].clone(),
    //         transform: ..default(),
    //     })
    //     .with_children(|parent| {
    //         parent
    //             .spawn_bundle(TransformBundle {
    //                 local: Transform::from_xyz(1.5, wall_height, 0.),
    //                 ..default()
    //             })
    //             .insert(Collider::cuboid(1.5, wall_height, 0.1));
    //     });
}

fn spawn_arch(
    commands: &mut Commands,
    position: Vec3,
    angle: Quat,
    wall_type: ArchType,
    model: &SharedModels,
) {
    if wall_type == ArchType::None {
        return;
    }

    model.arch.build(
        Transform::from_xyz(position.x, position.y, position.z).with_rotation(angle),
        commands.spawn(),
    );
}

///Is this tile accessible?
/// Both tops and bottoms of stairs are accessible
fn is_walkable(grid: &LevelGrid, xyz: (isize, isize, isize)) -> bool {
    match grid.get(xyz.0, xyz.1, xyz.2) {
        GridTile::Empty | GridTile::OutOfBounds => false,
        _ => true,
    }
}

/// Is there a accessible spot below this tile?
fn is_above_walkable(grid: &LevelGrid, xyz: (isize, isize, isize)) -> bool {
    for z in 0..=xyz.2 {
        if is_walkable(grid, (xyz.0, xyz.1, z)) {
            return true;
        }
    }
    return false;
}

/// Is this area covered by a roof?
fn is_indoor(grid: &LevelGrid, x: isize, y: isize) -> bool {
    grid.get_max_height(x, y) == 1
}

/// Is the area indoor but not above walkable
/// AKA is this area both unreachable and invisible
fn is_unreachable(grid: &LevelGrid, xyz: (isize, isize, isize)) -> bool {
    is_indoor(grid, xyz.0, xyz.1) && !is_above_walkable(grid, xyz)
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum WallType {
    None,
    Short,
    StairLeft,
    StairRight,
    Tall,
    Window,
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
    if !is_above_walkable(grid, p1)
        && !is_above_walkable(grid, p2)
        && (p2.2..LEVEL_SIZE.2 as isize)
            .any(|z| is_walkable(grid, (p1.0, p1.1, z)) != is_above_walkable(grid, (p2.0, p2.1, z)))
    {
        return WallType::Tall;
    }

    // Case 2: Edge of walkable area, but both sides are below walkable. This is a interior wall
    if is_above_walkable(grid, p1) ^ is_above_walkable(grid, p2)
        && !grid.can_access(p1, p2)
        && is_above_walkable(grid, (p1.0, p1.1, LEVEL_SIZE.2 as isize - 1))
        && is_above_walkable(grid, (p2.0, p2.1, LEVEL_SIZE.2 as isize - 1))
    {
        return WallType::Tall;
    }

    // Edge of building
    if is_unreachable(grid, p1) != is_unreachable(grid, p2) && p1.2 <= 2 {
        return WallType::Tall;
    }

    if is_indoor(grid, p1.0, p1.1) != is_indoor(grid, p2.0, p2.1)
        && is_above_walkable(grid, p1) != is_above_walkable(grid, p2)
        && !grid.can_access(p1, p2)
    {
        if !is_walkable(grid, p1) && !is_walkable(grid, p2) {
            return WallType::Window;
        } else {
            return WallType::Tall;
        }
    }

    if !(grid
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

    if corners.iter().all(|i| !is_above_walkable(grid, *i)) {
        return PillarType::None;
    }

    if corners.iter().any(|i| is_indoor(grid, i.0, i.1)) {
        return PillarType::Tall;
    }

    if corners
        .iter()
        .any(|i| is_walkable(grid, (i.0, i.1, i.2 + 1)))
    {
        return PillarType::Tall;
    }

    if corners.iter().any(|i| is_walkable(grid, *i))
        && !corners
            .iter()
            .all(|i| grid.get(i.0, i.1, i.2) == GridTile::Floor)
    {
        return PillarType::Short;
    }

    return PillarType::None;
}

fn add_floors(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
    // let floor_mesh: Handle<Mesh> = asset_server.load("Floor.glb#Mesh0/Primitive0");
    // let stairs_mesh: Handle<Mesh> = asset_server.load("Stairs.glb#Mesh0/Primitive0");
    // let stairs_bottom_mesh: Handle<Mesh> = asset_server.load("UnderStairs.glb#Mesh0/Primitive0");

    // let stair_collider = Collider::convex_hull(&[
    //     Vec3::new(1.5, 0.05, -1.5),
    //     Vec3::new(1.5, 0.05, 1.5),
    //     Vec3::new(-1.5, 2.6, -1.5),
    //     Vec3::new(-1.5, 2.6, 1.5),
    //     Vec3::new(-1.5, 0.05, -1.5),
    //     Vec3::new(-1.5, 0.05, 1.5),
    // ])
    // .unwrap();

    for (x, y, z) in grid {
        if grid.get(x, y, z) != GridTile::Empty && !grid.get(x, y, z).is_top_stair_tile() {
            let mesh = if grid.get(x, y, z).is_bottom_stair_tile() {
                &models.stairs
            } else {
                &models.floor
            };

            mesh.build(
                Transform::from_xyz(
                    x as f32 * LEVEL_SCALE.0,
                    z as f32 * LEVEL_SCALE.2,
                    y as f32 * LEVEL_SCALE.1,
                )
                .with_rotation(grid.get(x, y, z).get_angle()),
                commands.spawn(),
            );
            // commands
            //     .spawn_bundle(PbrBundle {
            //         mesh: if grid.get(x, y, z).is_bottom_stair_tile() {
            //             stairs_mesh.clone()
            //         } else {
            //             floor_mesh.clone()
            //         },
            //         material: if grid.get(x, y, z).is_stair_tile() {
            //             materials.generic.clone()
            //         } else {
            //             materials.floor.clone()
            //         },
            //         transform: Transform::from_xyz(
            //             x as f32 * LEVEL_SCALE.0,
            //             z as f32 * LEVEL_SCALE.2,
            //             y as f32 * LEVEL_SCALE.1,
            //         )
            //         .with_rotation(grid.get(x, y, z).get_angle()),
            //         // .with_rotation(Quat::from_rotation_x(
            //         //     ((rng.gen::<f32>() - 0.5) * 20.0_f32).to_radians(),
            //         // )),
            //         ..default()
            //     })
            //     .insert(if grid.get(x, y, z).is_bottom_stair_tile() {
            //         stair_collider.clone()
            //     } else {
            //         Collider::cuboid(1.5, 0.1, 1.5)
            //     });

            if grid.get(x, y, z).is_stair_tile() {
                models.under_stairs.build(
                    Transform::from_xyz(
                        x as f32 * LEVEL_SCALE.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1,
                    )
                    .with_rotation(grid.get(x, y, z).get_angle()),
                    commands.spawn(),
                )
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum ArchType {
    None,
    Normal,
}

fn should_build_arch(
    grid: &LevelGrid,
    t1: (isize, isize, isize),
    t2: (isize, isize, isize),
) -> ArchType {
    if matches!(
        should_build_wall(grid, t1, t2),
        WallType::Tall | WallType::Window
    ) {
        return ArchType::None;
    }

    if !is_above_walkable(grid, t1) && !is_above_walkable(grid, t2) {
        return ArchType::None;
    }

    if t1.2 == LEVEL_SIZE.2 as isize - 1
        && (is_indoor(grid, t1.0, t1.1) || is_indoor(grid, t2.0, t2.1))
    {
        return ArchType::Normal;
    }

    if (is_walkable(grid, (t1.0, t1.1, t1.2 + 1))
        && !grid.get(t1.0, t1.1, t1.2 + 1).is_top_stair_tile())
        || (is_walkable(grid, (t2.0, t2.1, t2.2 + 1))
            && !grid.get(t2.0, t2.1, t2.2 + 1).is_top_stair_tile())
    {
        return ArchType::Normal;
    }

    return ArchType::None;
}

fn add_roofs(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
    for x in 0..LEVEL_SIZE.0 {
        for y in 0..LEVEL_SIZE.1 {
            if grid.1[x][y] == 1 {
                models.roof.build(
                    Transform::from_xyz(
                        x as f32 * LEVEL_SCALE.0,
                        ROOF_HEIGHT as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1,
                    )
                    .with_rotation(Quat::from_rotation_y(if y % 2 == 0 {
                        consts::PI
                    } else {
                        0.
                    })),
                    commands.spawn(),
                )
                // for (model, material) in roof_model.iter().zip(materials.iter()) {

                //     commands.spawn_bundle(PbrBundle {
                //         mesh: model.clone(),
                //         material: material.clone(),
                //         transform: Transform::from_xyz(
                //             x as f32 * LEVEL_SCALE.0,
                //             ROOF_HEIGHT as f32 * LEVEL_SCALE.2,
                //             y as f32 * LEVEL_SCALE.1,
                //         )
                //         .with_rotation(Quat::from_rotation_y(
                //             if y % 2 == 0 { consts::PI } else { 0. },
                //         )),
                //         ..default()
                //     });
                // }
            }
        }
    }
}

fn add_walls(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
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
                    models,
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
                    models,
                );
            }
        }
    }
}

fn build_arches(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in -1..LEVEL_SIZE.2 as isize {
                spawn_arch(
                    commands,
                    Vec3::new(
                        x as f32 * LEVEL_SCALE.0 - LEVEL_SCALE.0 / 2.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1,
                    ),
                    Quat::default(),
                    should_build_arch(grid, (x - 1, y, z), (x, y, z)),
                    models,
                );

                spawn_arch(
                    commands,
                    Vec3::new(
                        x as f32 * LEVEL_SCALE.0,
                        z as f32 * LEVEL_SCALE.2,
                        y as f32 * LEVEL_SCALE.1 - LEVEL_SCALE.1 / 2.0,
                    ),
                    Quat::from_rotation_y(-consts::FRAC_PI_2),
                    should_build_arch(grid, (x, y - 1, z), (x, y, z)),
                    models,
                );
            }
        }
    }
}

fn add_pillars(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
    for x in 0..LEVEL_SIZE.0 as isize + 1 {
        for y in 0..LEVEL_SIZE.1 as isize + 1 {
            for z in -1..LEVEL_SIZE.2 as isize {
                let edges = should_build_pillar(grid, (x, y, z));
                if edges != PillarType::None {
                    let model = if edges == PillarType::Tall {
                        &models.pillar_tall
                    } else {
                        &models.pillar_short
                    };

                    model.build(
                        Transform::from_xyz(
                            x as f32 * 3. - 1.5,
                            z as f32 * 2.5,
                            y as f32 * 3. - 1.5,
                        ),
                        commands.spawn(),
                    );

                    // commands.spawn_bundle(PbrBundle {
                    //     mesh: if edges == PillarType::Tall {
                    //         pillar_tall_model.clone()
                    //     } else {
                    //         pillar_short_model.clone()
                    //     },
                    //     material: materials.generic.clone(),
                    //     transform: Transform::from_xyz(
                    //         x as f32 * 3. - 1.5,
                    //         z as f32 * 2.5,
                    //         y as f32 * 3. - 1.5,
                    //     ),
                    //     ..default()
                    // });
                }
            }
        }
    }
}

fn add_vases(
    grid: &LevelGrid,
    commands: &mut Commands,
    shared_materials: &SharedMaterials,
    asset_server: &AssetServer,
) {
    let vases: [Handle<Mesh>; 3] = [
        asset_server.load("Vase1.glb#Mesh0/Primitive0"),
        asset_server.load("Vase2.glb#Mesh0/Primitive0"),
        asset_server.load("Vase3.glb#Mesh0/Primitive0"),
    ];

    let mut rng = rand::thread_rng();
    for (x, y, z) in grid {
        if grid.get(x, y, z) == GridTile::Floor {
            if rng.gen_range(0.0..1.0) > 0.8 {
                commands.spawn_bundle(PbrBundle {
                    mesh: vases[rng.gen_range(0..3)].clone(),
                    material: shared_materials.generic.clone(),
                    transform: Transform::from_xyz(
                        x as f32 * 3.0 + rng.gen_range(-1.3..1.3),
                        z as f32 * 2.5,
                        y as f32 * 3.0 + rng.gen_range(-1.3..1.3),
                    ),
                    ..default()
                });
            }
        }
    }
}

fn add_lights(grid: &LevelGrid, commands: &mut Commands, models: &SharedModels) {
    for pos in grid {
        if (pos.0 % 3 == pos.1 % 3)
            && pos.2 == LEVEL_SIZE.2 as isize - 1
            && is_indoor(grid, pos.0, pos.1)
            && is_above_walkable(grid, pos)
        {
            commands.spawn_bundle(PointLightBundle {
                transform: Transform::from_xyz(
                    pos.0 as f32 * LEVEL_SCALE.0,
                    pos.2 as f32 * LEVEL_SCALE.2,
                    pos.1 as f32 * LEVEL_SCALE.1,
                ),
                point_light: PointLight {
                    intensity: 200.0,
                    range: 7.5,
                    ..default()
                },
                ..default()
            });

            models.lamp.build(
                Transform::from_xyz(
                    pos.0 as f32 * LEVEL_SCALE.0,
                    pos.2 as f32 * LEVEL_SCALE.2,
                    pos.1 as f32 * LEVEL_SCALE.1,
                ),
                commands.spawn(),
            )
        }
    }
}

pub(super) fn decorate_level(
    grid: &LevelGrid,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let loading_start = std::time::Instant::now();
    let shared_materials = SharedMaterials::new(materials, &asset_server);
    let shared_models = SharedModels::new(&shared_materials, &asset_server);

    let start_time = std::time::Instant::now();

    add_floors(grid, commands, &shared_models);
    add_roofs(grid, commands, &shared_models);
    add_walls(grid, commands, &shared_models);
    add_pillars(grid, commands, &shared_models);
    add_vases(grid, commands, &shared_materials, &asset_server);
    build_arches(grid, commands, &shared_models);
    add_lights(grid, commands, &shared_models);

    let end_time = std::time::Instant::now();
    let difference = end_time - start_time;
    println!(
        "Creating the standard model list look {:?}",
        start_time - loading_start
    );
    println!("Level decoration look {:?}", difference);
}
