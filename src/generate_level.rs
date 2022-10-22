use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use std::f32::consts;

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

fn generate_options(
    grid: &LevelGrid,
    cursor_position: (usize, usize, usize),
) -> Vec<(GridTile, (usize, usize), Option<(usize, usize)>)> {
    let mut options = Vec::new();

    for cardinal_direction in [
        (-1, 0, GridTile::StairsWestTop, GridTile::StairsEastBottom),
        (0, -1, GridTile::StairsNorthTop, GridTile::StairsSouthBottom),
        (1, 0, GridTile::StairsEastTop, GridTile::StairsWestBottom),
        (0, 1, GridTile::StairsSouthTop, GridTile::StairsNorthBottom),
    ] {
        if cardinal_direction.0 + (cursor_position.0 as isize) < 0
            || cardinal_direction.1 + (cursor_position.1 as isize) < 0
        {
            continue;
        }

        let offset_tile_position = (
            (cardinal_direction.0 + cursor_position.0 as isize) as usize,
            (cardinal_direction.1 + cursor_position.1 as isize) as usize,
        );

        if offset_tile_position.0 >= grid.len() || offset_tile_position.1 >= grid[0].len() {
            continue;
        }

        if grid[offset_tile_position.0][offset_tile_position.1][cursor_position.2]
            == GridTile::Empty
        {
            options.push((GridTile::Floor, offset_tile_position, None));

            if (offset_tile_position.0 as isize + cardinal_direction.0 > 0
                && offset_tile_position.1 as isize + cardinal_direction.1 > 0
                && offset_tile_position.0 as isize + cardinal_direction.0 < grid.len() as isize
                && offset_tile_position.1 as isize + cardinal_direction.1 < grid[0].len() as isize)
            {
                let expanded_grid_tile = (
                    (offset_tile_position.0 as isize + cardinal_direction.0) as usize,
                    (offset_tile_position.1 as isize + cardinal_direction.1) as usize,
                );

                if cursor_position.2 > 0
                    && grid[offset_tile_position.0][offset_tile_position.1][cursor_position.2 - 1]
                        == GridTile::Empty
                    && grid[expanded_grid_tile.0][expanded_grid_tile.1][cursor_position.2 - 1]
                        == GridTile::Empty
                {
                    options.push((
                        cardinal_direction.2,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                }

                if cursor_position.2 < 2
                    && grid[offset_tile_position.0][offset_tile_position.1][cursor_position.2 + 1]
                        == GridTile::Empty
                    && grid[expanded_grid_tile.0][expanded_grid_tile.1][cursor_position.2 + 1]
                        == GridTile::Empty
                {
                    options.push((
                        cardinal_direction.3,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                }
            }
        }
    }

    return options;
}

fn generate_level_grid() -> Box<LevelGrid> {
    let mut grid = [[[GridTile::Empty; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0];

    let mut cursor_position = (6, 6, 1);
    grid[cursor_position.0][cursor_position.1][cursor_position.2] = GridTile::Floor;

    let mut rng = rand::thread_rng();

    loop {
        let options = generate_options(&grid, cursor_position);

        println!("> {:?}", cursor_position);
        match options.choose(&mut rng) {
            Some((new_tile_type, new_tile_position, new_new_tile_position)) => {
                assert_eq!(
                    grid[new_tile_position.0][new_tile_position.1][cursor_position.2],
                    GridTile::Empty
                );
                grid[new_tile_position.0][new_tile_position.1][cursor_position.2] = *new_tile_type;
                cursor_position.0 = new_tile_position.0;
                cursor_position.1 = new_tile_position.1;
                if new_tile_type.is_stair_tile() {
                    if new_tile_type.is_bottom_stair_tile() {
                        cursor_position.2 += 1;
                    } else if new_tile_type.is_top_stair_tile() {
                        cursor_position.2 -= 1;
                    }
                    println!("# {:?}", cursor_position);
                    grid[new_tile_position.0][new_tile_position.1][cursor_position.2] =
                        new_tile_type.get_opposite_stair_tile().unwrap();

                    let new_new_tile_position = new_new_tile_position.unwrap();
                    cursor_position.0 = new_new_tile_position.0;
                    cursor_position.1 = new_new_tile_position.1;

                    grid[cursor_position.0][cursor_position.1][cursor_position.2] = GridTile::Floor;
                }
            }
            None => break,
        }
    }

    return Box::new(grid);
}

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

fn decorate_level(
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
                                x as f32 * 3.0,
                                z as f32 * 2.5,
                                y as f32 * 3.0,
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

pub fn generate_level(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let grid = generate_level_grid();
    decorate_level(grid, commands, meshes, materials, asset_server);
}
