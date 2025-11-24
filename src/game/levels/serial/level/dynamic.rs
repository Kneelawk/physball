use crate::game::assets::asset_ref;
use crate::game::levels::LevelObject;
use crate::game::levels::button::ButtonPresser;
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
pub struct SerialDynamicObject {
    pub ty: DynamicObjectType,
    pub dimensions: Vec3,
    pub trans: Transform,
    pub material: Handle<StandardMaterial>,
}

#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Reflect, strum::VariantArray, strum::Display,
)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum DynamicObjectType {
    #[default]
    Sphere,
    Cube,
}

impl SerialDynamicObject {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let ty = node
            .get_variant("type", DynamicObjectType::VARIANTS, &source)
            .map(|ty| ty.copied().unwrap_or_default());

        let material = node
            .get_handle("material", load_context, &source)
            .map(|handle| {
                handle.unwrap_or_else(|| asset_ref::default_plane_material(load_context))
            });

        let trans = node
            .children()
            .map_or(Ok(None), |children| {
                children.get_transform(&source).map(Some)
            })
            .map(|trans| trans.unwrap_or_default());

        let dimensions = node
            .children()
            .and_then(|children| children.get("size"))
            .map_or(Ok(None), |size| size.must_get_scale(0, &source).map(Some))
            .map(|size| size.unwrap_or(Vec3::splat(0.25)));

        let (ty, material, trans, dimensions) = (ty, material, trans, dimensions).merge()?;

        Ok(Self {
            ty,
            dimensions,
            trans,
            material,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        if args.dyn_assets {
            args.cmd.spawn((
                LevelObject,
                self.trans,
                ButtonPresser,
                Mesh3d(args.assets.add(self.ty.to_mesh(self.dimensions))),
                MeshMaterial3d(self.material.clone()),
                RigidBody::Dynamic,
                self.ty.to_collider(self.dimensions),
            ));
        }
    }
}

impl DynamicObjectType {
    pub fn to_mesh(self, dimensions: Vec3) -> Mesh {
        match self {
            DynamicObjectType::Sphere => Sphere::new(dimensions.x).into(),
            DynamicObjectType::Cube => Cuboid::from_size(dimensions).into(),
        }
    }

    pub fn to_collider(self, dimensions: Vec3) -> Collider {
        match self {
            DynamicObjectType::Sphere => Collider::sphere(dimensions.x),
            DynamicObjectType::Cube => Collider::cuboid(dimensions.x, dimensions.y, dimensions.z),
        }
    }
}
