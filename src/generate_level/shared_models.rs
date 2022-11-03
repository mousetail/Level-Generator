use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub(super) struct SharedMaterials {
    pub floor: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub generic: Handle<StandardMaterial>,
    pub roof: Handle<StandardMaterial>,
    pub wood: Handle<StandardMaterial>,
    pub marble: Handle<StandardMaterial>,
}

impl SharedMaterials {
    pub fn new(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) -> SharedMaterials {
        SharedMaterials {
            floor: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Marble_TilesDiagonal2_512_albedo.png"),
                ),
                metallic_roughness_texture: Some(
                    asset_server.load("textures/Marble_tilesDiagonal2_512_roughness.png.norm"),
                ),
                perceptual_roughness: 1.0,
                ..default()
            }),
            wall: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Plaster_Plain_512_albedo.png"),
                ),
                normal_map_texture: Some(
                    asset_server.load("textures/Plaster_Plain_1k_normal.png.norm"),
                ),
                ..default()
            }),
            generic: materials.add(Color::rgb(1., 1., 1.).into()),
            roof: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Roofing_SquareOld2_1K_albedo.png"),
                ),
                normal_map_texture: Some(
                    asset_server.load("textures/Roofing_SquareOld2_1K_normal.png.norm"),
                ),
                ..default()
            }),
            wood: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Wood_PlanksTemple1_3x3_1K_albedo.png"),
                ),
                normal_map_texture: Some(
                    asset_server.load("textures/Wood_PlanksTemple1_3x3_1K_normal.png.norm"),
                ),
                ..default()
            }),
            marble: materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load("textures/Marble_SlabWhite2_512_albedo.png"),
                ),
                ..default()
            }),
        }
    }
}

struct SimpleModelConponent {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl SimpleModelConponent {
    fn spawn(&self, transform: Transform) -> PbrBundle {
        PbrBundle {
            mesh: self.mesh.clone(),
            material: self.material.clone(),
            transform: transform,
            ..default()
        }
    }
}

pub struct SimpleModel {
    components: Vec<SimpleModelConponent>,
    collider: Option<(Collider, Vec3)>,
}

impl SimpleModel {
    fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> SimpleModel {
        return SimpleModelConponent {
            mesh: mesh,
            material: material,
        }
        .into();
    }

    fn new_with_collider(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        collider: Collider,
    ) -> SimpleModel {
        Self::new_with_offset_collider(mesh, material, collider, Vec3::ZERO)
    }

    fn new_with_offset_collider(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        collider: Collider,
        offset: Vec3,
    ) -> SimpleModel {
        return SimpleModel {
            components: vec![SimpleModelConponent {
                mesh: mesh,
                material: material,
            }],
            collider: Some((collider, offset)),
        };
    }

    pub fn build(&self, transform: Transform, mut commands: bevy::ecs::system::EntityCommands) {
        commands
            .insert_bundle(self.components[0].spawn(transform))
            .with_children(|parent| {
                for component in &self.components[1..] {
                    parent
                        .spawn()
                        .insert_bundle(component.spawn(Transform::default()));
                }
                if let Some((collider, offset)) = &self.collider {
                    parent
                        .spawn_bundle(TransformBundle {
                            local: Transform::from_translation(*offset),
                            ..default()
                        })
                        .insert(collider.clone());
                }
            });
    }
}

impl From<SimpleModelConponent> for SimpleModel {
    fn from(e: SimpleModelConponent) -> SimpleModel {
        SimpleModel {
            components: vec![e],
            collider: None,
        }
    }
}

pub(super) struct SharedModels {
    pub arch: SimpleModel,
    pub floor: SimpleModel,
    pub pillar_short: SimpleModel,
    pub pillar_tall: SimpleModel,
    pub railing: SimpleModel,
    pub roof: SimpleModel,
    pub stair_railing: SimpleModel,
    pub stairs: SimpleModel,
    pub under_stairs: SimpleModel,
    pub wall: SimpleModel,
    pub wall_window: SimpleModel,
    pub lamp: SimpleModel,
}

impl SharedModels {
    pub fn new(materials: &SharedMaterials, asset_server: &Res<AssetServer>) -> SharedModels {
        SharedModels {
            arch: SimpleModel::new(
                asset_server.load("Arch.glb#Mesh0/Primitive0"),
                materials.wood.clone(),
            ),
            floor: SimpleModel::new_with_offset_collider(
                asset_server.load("Floor.glb#Mesh0/Primitive0"),
                materials.floor.clone(),
                Collider::cuboid(1.5, 0.1, 1.5),
                Vec3::new(0.0, -0.1, 0.0),
            ),
            pillar_short: SimpleModel::new(
                asset_server.load("PillarShort.glb#Mesh0/Primitive0"),
                materials.generic.clone(),
            ),
            pillar_tall: SimpleModel::new_with_offset_collider(
                asset_server.load("PillarTall.glb#Mesh0/Primitive0"),
                materials.generic.clone(),
                Collider::cuboid(0.1, 1.5, 0.1),
                Vec3::new(0., 1.5, 0.),
            ),
            railing: SimpleModel::new_with_offset_collider(
                asset_server.load("Railing.glb#Mesh0/Primitive0"),
                materials.marble.clone(),
                Collider::cuboid(1.5, 0.75 / 2., 0.1),
                Vec3::new(1.5, 0.75 / 2., 0.0),
            ),
            roof: SimpleModel {
                components: vec![
                    SimpleModelConponent {
                        mesh: asset_server.load("Roof.glb#Mesh0/Primitive0"),
                        material: materials.floor.clone(),
                    },
                    SimpleModelConponent {
                        mesh: asset_server.load("Roof.glb#Mesh0/Primitive1"),
                        material: materials.wall.clone(),
                    },
                    SimpleModelConponent {
                        mesh: asset_server.load("Roof.glb#Mesh0/Primitive2"),
                        material: materials.roof.clone(),
                    },
                ],
                collider: None,
            },
            stair_railing: SimpleModel::new_with_collider(
                asset_server.load("StairRailing.glb#Mesh0/Primitive0"),
                materials.marble.clone(),
                Collider::convex_hull(&[
                    Vec3::new(0.0, 0.0, -0.1),
                    Vec3::new(0.0, 0.0, 0.1),
                    Vec3::new(0.0, 0.75, -0.1),
                    Vec3::new(0.0, 0.75, 0.1),
                    Vec3::new(3.0, 2.5, -0.1),
                    Vec3::new(3.0, 2.5, 0.1),
                    Vec3::new(3.0, 3.25, -0.1),
                    Vec3::new(3.0, 3.25, 0.1),
                ])
                .unwrap(),
            ),
            stairs: SimpleModel::new_with_collider(
                asset_server.load("Stairs.glb#Mesh0/Primitive0"),
                materials.generic.clone(),
                Collider::convex_hull(&[
                    Vec3::new(1.5, 0.00, -1.5),
                    Vec3::new(1.5, 0.00, 1.5),
                    Vec3::new(-1.5, 2.5, -1.5),
                    Vec3::new(-1.5, 2.5, 1.5),
                    Vec3::new(-1.5, 0.00, -1.5),
                    Vec3::new(-1.5, 0.00, 1.5),
                ])
                .unwrap(),
            ),
            under_stairs: SimpleModel::new(
                asset_server.load("UnderStairs.glb#Mesh0/Primitive0"),
                materials.wall.clone(),
            ),
            wall: SimpleModel::new_with_offset_collider(
                asset_server.load("Wall.glb#Mesh0/Primitive0"),
                materials.wall.clone(),
                Collider::cuboid(1.5, 2.5 / 2., 0.1),
                Vec3::new(1.5, 2.5 / 2., 0.0),
            ),
            wall_window: SimpleModel {
                components: vec![
                    SimpleModelConponent {
                        mesh: asset_server.load("Window.glb#Mesh0/Primitive0"),
                        material: materials.wall.clone(),
                    },
                    SimpleModelConponent {
                        mesh: asset_server.load("Window.glb#Mesh0/Primitive1"),
                        material: materials.wood.clone(),
                    },
                ],
                collider: None,
            },
            lamp: SimpleModel::new(
                asset_server.load("Lamp.glb#Mesh0/Primitive0"),
                materials.generic.clone(),
            ),
        }
    }
}
