use crate::game::assets::preload::Preloads;
use crate::game::levels::finish_point::FinishPoint;
use crate::game::levels::{LevelObject, PlayerSpawnPoint};
use crate::type_expr;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAtlas};

pub struct LevelBuildArgs<'a, 'w, 's> {
    /// Whether to spawn the assets that would otherwise be spawned by a checkpoint load
    pub dyn_assets: bool,
    pub cmd: &'a mut Commands<'w, 's>,
    pub assets: &'a AssetServer,
    pub preloads: &'a Preloads,
}

#[derive(Debug, Clone, Asset, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialLevel {
    pub spawn: Transform,
    pub finish: Transform,
    pub planes: Vec<SerialPlane>,
    pub texts: Vec<SerialText>,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialPlane {
    pub width: f32,
    pub length: f32,
    pub trans: Transform,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialText {
    pub text: String,
    pub trans: Transform,
}

impl SerialLevel {
    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((LevelObject, PlayerSpawnPoint, self.spawn));
        args.cmd.spawn((LevelObject, FinishPoint, self.finish));

        for plane in self.planes.iter() {
            plane.spawn(args);
        }

        for text in self.texts.iter() {
            text.spawn(args);
        }
    }
}

impl SerialPlane {
    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((
            LevelObject,
            self.trans,
            Mesh3d(
                args.assets.add(
                    Plane3d::new(Vec3::Y, Vec2::new(self.width / 2.0, self.length / 2.0)).into(),
                ),
            ),
            MeshMaterial3d(
                args.assets
                    .add(type_expr!(StandardMaterial, Color::WHITE.into())),
            ),
            children![(
                RigidBody::Static,
                Collider::cuboid(self.width, 0.2, self.length),
                Transform::from_xyz(0.0, -0.1, 0.0)
            )],
        ));
    }
}

impl SerialText {
    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((
            LevelObject,
            self.trans,
            Text3d::new(self.text.clone()),
            Text3dStyling {
                font: "sans-serif".into(),
                size: 64.,
                world_scale: Some(Vec2::splat(0.25)),
                ..Default::default()
            },
            Mesh3d::default(),
            MeshMaterial3d(args.assets.add(StandardMaterial {
                base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                emissive: LinearRgba::new(0.0, 10.0, 12.0, 1.0),
                ..default()
            })),
        ));
    }
}
