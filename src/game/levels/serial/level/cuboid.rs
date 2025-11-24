use crate::game::assets::asset_ref;
use crate::game::levels::LevelObject;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::serial::level::LevelBuildArgs;
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use kdl::KdlNode;
use std::sync::Arc;

pub struct SerialCuboid {
    pub material: Handle<StandardMaterial>,
    pub dimensions: Vec3,
    pub trans: Transform,
}

impl SerialCuboid {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let material = node
            .get_handle("material", load_context, &source)
            .map(|handle| {
                handle.unwrap_or_else(|| asset_ref::default_plane_material(load_context))
            });

        let dimensions = node
            .must_children(&source)
            .and_then(|doc| doc.must_get("size", &source))
            .and_then(|node| node.must_get_scale(0, &source));

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (material, dimensions, trans) = (material, dimensions, trans).merge()?;

        Ok(Self {
            material,
            dimensions,
            trans,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((
            LevelObject,
            self.trans,
            Mesh3d(args.assets.add(Cuboid::from_size(self.dimensions).into())),
            MeshMaterial3d(self.material),
            RigidBody::Static,
            Collider::cuboid(self.dimensions.x, self.dimensions.y, self.dimensions.z),
        ));
    }
}
