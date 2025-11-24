use crate::game::assets::asset_ref;
use crate::game::levels::LevelObject;
use crate::game::levels::death::DeathCollider;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::serial::level::LevelBuildArgs;
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use kdl::KdlNode;
use std::sync::Arc;
use strum::VariantArray;

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialPlane {
    pub width: f32,
    pub length: f32,
    pub trans: Transform,
    pub material: Handle<StandardMaterial>,
    pub ty: SerialPlaneType,
}

#[derive(Debug, Default, Copy, Clone, Reflect, strum::VariantArray, strum::Display)]
#[reflect(Debug, Default, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum SerialPlaneType {
    #[default]
    Static,
    Death,
}

impl SerialPlane {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let size = node.must_get_number(0, &source);
        let size2 = node.get_number(1, &source);

        let ty = node
            .get_variant("type", SerialPlaneType::VARIANTS, &source)
            .map(|ty| ty.copied().unwrap_or_default());

        let material = node
            .get_handle("material", load_context, &source)
            .map(|handle| {
                handle.unwrap_or_else(|| asset_ref::default_plane_material(load_context))
            });

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (size, size2, ty, material, trans) = (size, size2, ty, material, trans).merge()?;

        Ok(SerialPlane {
            width: size as f32,
            length: size2.unwrap_or(size) as f32,
            trans,
            material,
            ty,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        match self.ty {
            SerialPlaneType::Static => {
                args.cmd.spawn((
                    LevelObject,
                    self.trans,
                    Mesh3d(
                        args.assets.add(
                            Plane3d::new(Vec3::Y, Vec2::new(self.width / 2.0, self.length / 2.0))
                                .into(),
                        ),
                    ),
                    MeshMaterial3d(self.material.clone()),
                    children![(
                        RigidBody::Static,
                        Collider::cuboid(self.width, 0.2, self.length),
                        Transform::from_xyz(0.0, -0.1, 0.0)
                    )],
                ));
            }
            SerialPlaneType::Death => {
                args.cmd.spawn((
                    LevelObject,
                    self.trans
                        .with_translation(self.trans.translation + vec3(0.0, -0.1, 0.0)),
                    RigidBody::Static,
                    Collider::cuboid(self.width, 0.2, self.length),
                    DeathCollider,
                ));
            }
        }
    }
}
