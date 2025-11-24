use crate::game::assets::asset_ref;
use crate::game::levels::button::{ControlledBy, LevelButton, LevelButtonDoor};
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::serial::level::LevelBuildArgs;
use avian3d::prelude::{Collider, RigidBody};
use bevy::asset::{Handle, LoadContext};
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude;
use bevy::prelude::{Cuboid, Entity, Reflect, Transform};
use kdl::KdlNode;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialButton {
    pub name: String,
    pub trans: Transform,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialButtonDoor {
    pub name: String,
    pub trans: Transform,
    pub open_trans: Transform,
    pub dimensions: Vec3,
    pub material: Handle<StandardMaterial>,
}

impl SerialButton {
    pub fn bind(
        node: &KdlNode,
        _load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> prelude::Result<Self, KdlBindError> {
        let name = node
            .must_get_string(0, &source)
            .map(|name| name.to_string());

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (name, trans) = (name, trans).merge()?;

        Ok(Self { name, trans })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) -> Entity {
        args.cmd.spawn((LevelButton, self.trans)).id()
    }
}

impl SerialButtonDoor {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> prelude::Result<Self, KdlBindError> {
        let name = node
            .must_get_string(0, &source)
            .map(|name| name.to_string());

        let trans = node
            .must_children(&source)
            .and_then(|children| children.must_children("default", &source))
            .and_then(|default| default.get_transform(&source));

        let open_trans = node
            .must_children(&source)
            .and_then(|children| children.must_children("open", &source))
            .and_then(|open| open.get_transform(&source));

        let dimensions = node
            .must_child("size", &source)
            .and_then(|child| child.must_get_scale(0, &source));

        let material = node
            .get_handle("material", load_context, &source)
            .map(|handle| {
                handle.unwrap_or_else(|| asset_ref::default_plane_material(load_context))
            });

        let (name, trans, open_trans, dimensions, material) =
            (name, trans, open_trans, dimensions, material).merge()?;

        Ok(Self {
            name,
            trans,
            open_trans,
            dimensions,
            material,
        })
    }

    pub fn spawn(&self, button_names: &HashMap<String, Entity>, args: &mut LevelBuildArgs) {
        let mut commands = args.cmd.spawn((
            LevelButtonDoor {
                default_trans: self.trans,
                open_trans: self.open_trans,
                openness: 0.0,
            },
            Mesh3d(args.assets.add(Cuboid::from_size(self.dimensions).into())),
            MeshMaterial3d(self.material.clone()),
            RigidBody::Kinematic,
            Collider::cuboid(self.dimensions.x, self.dimensions.y, self.dimensions.z),
        ));
        if let Some(button) = button_names.get(&self.name) {
            commands.insert(ControlledBy(*button));
        }
    }
}
