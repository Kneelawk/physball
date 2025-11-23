use crate::capture_result;
use crate::game::assets::asset_ref;
use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::levels::button::{ButtonPresser, ControlledBy, LevelButton, LevelButtonDoor};
use crate::game::levels::death::DeathCollider;
use crate::game::levels::finish_point::FinishPoint;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::{LevelObject, PlayerSpawnPoint};
use crate::game::music;
use crate::game::music::{BackgroundMusic, BackgroundMusicTrigger};
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign};
use kdl::{KdlDocument, KdlNode};
use std::collections::HashMap;
use std::string::ToString;
use std::sync::Arc;
use strum::VariantArray;

pub const DEFAULT_TEXT_PT: f64 = 64.0;

pub struct LevelBuildArgs<'a, 'w, 's> {
    /// Whether to spawn the assets that would otherwise be spawned by a checkpoint load
    pub dyn_assets: bool,
    pub cmd: &'a mut Commands<'w, 's>,
    pub assets: &'a AssetServer,
    pub preloads: &'a Preloads,
    pub fonts: &'a FontNames,
}

#[derive(Debug, Clone, Asset, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialLevel {
    pub spawn: Transform,
    pub finish: Transform,
    pub default_music: Option<SerialMusic>,
    pub triggered_music: Vec<SerialTriggeredMusic>,
    pub planes: Vec<SerialPlane>,
    pub texts: Vec<SerialText>,
    pub buttons: Vec<SerialButton>,
    pub button_doors: Vec<SerialButtonDoor>,
    pub dynamic_objects: Vec<SerialDynamicObject>,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialMusic {
    pub audio: Handle<AudioSource>,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialTriggeredMusic {
    pub audio: Handle<AudioSource>,
    pub dimensions: Vec3,
    pub trans: Transform,
}

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

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct SerialText {
    pub text: String,
    pub trans: Transform,
    pub material: Handle<StandardMaterial>,
    pub pt: f32,
    pub font: Handle<Font>,
    pub align: SerialAlign,
}

#[derive(Debug, Default, Copy, Clone, Reflect, strum::VariantArray, strum::Display)]
#[reflect(Debug, Default, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum SerialAlign {
    Left,
    #[default]
    Center,
    Right,
}

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

impl SerialLevel {
    pub fn bind(
        doc: &KdlDocument,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let spawn = capture_result! {
            let doc = doc.must_children("spawn", &source)?;
            doc.get_transform(&source)
        };

        let finish = capture_result! {
            let doc = doc.must_children("finish", &source)?;
            doc.get_transform(&source)
        };

        let default_music = doc
            .nodes()
            .iter()
            .find(|node| node.name().value() == "music")
            .map_or(Ok(None), |node| {
                SerialMusic::bind(node, load_context, source.clone()).map(Some)
            });

        let triggered_music = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "triggered_music")
            .map(|node| SerialTriggeredMusic::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let planes = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "plane")
            .map(|node| SerialPlane::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let texts = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "text")
            .map(|node| SerialText::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let buttons = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "button")
            .map(|node| SerialButton::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let button_doors = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "button_door")
            .map(|node| SerialButtonDoor::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let dynamic_objects = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "dyn")
            .map(|node| SerialDynamicObject::bind(node, load_context, source.clone()))
            .collect::<Vec<_>>()
            .merge();

        let (
            spawn,
            finish,
            default_music,
            triggered_music,
            planes,
            texts,
            buttons,
            button_doors,
            dynamic_objects,
        ) = (
            spawn,
            finish,
            default_music,
            triggered_music,
            planes,
            texts,
            buttons,
            button_doors,
            dynamic_objects,
        )
            .merge()?;

        Ok(Self {
            spawn,
            default_music,
            triggered_music,
            finish,
            planes,
            texts,
            buttons,
            button_doors,
            dynamic_objects,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((LevelObject, PlayerSpawnPoint, self.spawn));
        args.cmd.spawn((LevelObject, FinishPoint, self.finish));

        if let Some(default_music) = &self.default_music {
            default_music.spawn(args);
        }

        for triggered_music in self.triggered_music.iter() {
            triggered_music.spawn(args);
        }

        for plane in self.planes.iter() {
            plane.spawn(args);
        }

        for text in self.texts.iter() {
            text.spawn(args);
        }

        let mut button_names = HashMap::new();
        for button in self.buttons.iter() {
            button_names.insert(button.name.clone(), button.spawn(args));
        }

        for button_door in self.button_doors.iter() {
            button_door.spawn(&button_names, args);
        }

        for dynamic_object in self.dynamic_objects.iter() {
            dynamic_object.spawn(args);
        }
    }
}

impl SerialMusic {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let audio = node.must_get_handle(0, load_context, &source)?;
        Ok(Self { audio })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((
            LevelObject,
            BackgroundMusic(self.audio.id()),
            music::PLAYBACK_SETTINGS,
            AudioPlayer(self.audio.clone()),
        ));
    }
}

impl SerialTriggeredMusic {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let audio = node.must_get_handle(0, load_context, &source);

        let dimensions = node
            .must_children(&source)
            .and_then(|doc| doc.must_get("size", &source))
            .and_then(|node| node.must_get_scale(0, &source));

        let trans = node
            .must_children(&source)
            .and_then(|doc| doc.get_transform(&source));

        let (audio, dimensions, trans) = (audio, dimensions, trans).merge()?;

        Ok(Self {
            audio,
            dimensions,
            trans,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        args.cmd.spawn((
            LevelObject,
            self.trans,
            BackgroundMusicTrigger(self.audio.clone()),
            Sensor,
            Collider::cuboid(self.dimensions.x, self.dimensions.y, self.dimensions.z),
        ));
    }
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

impl SerialText {
    pub fn bind(
        node: &KdlNode,
        load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
        let text = node
            .must_get_string(0, &source)
            .map(|text| text.to_string());

        let pt = node
            .get_number("pt", &source)
            .map(|pt| pt.unwrap_or(DEFAULT_TEXT_PT));

        let font = node
            .get_handle::<Font>("font", load_context, &source)
            .map(|asset| asset.unwrap_or_else(|| asset_ref::default_font(load_context)));

        let align = node
            .get_variant("align", SerialAlign::VARIANTS, &source)
            .map(|align| align.copied().unwrap_or_default());

        let material = node
            .get_handle("material", load_context, &source)
            .map(|asset| asset.unwrap_or_else(|| asset_ref::default_text_material(load_context)));

        let trans = node
            .children()
            .map_or(Ok(None), |doc| doc.get_transform(&source).map(Some))
            .map(|trans| trans.unwrap_or_default());

        let (text, pt, font, align, material, trans) =
            (text, pt, font, align, material, trans).merge()?;

        Ok(Self {
            text,
            trans,
            material,
            pt: pt as f32,
            font,
            align,
        })
    }

    pub fn spawn(&self, args: &mut LevelBuildArgs) {
        let font = args
            .fonts
            .get(&self.font.id())
            .unwrap_or_else(|| {
                warn!("Font {:?} has not loaded yet, using default", &self.font);
                &args.fonts[&args.preloads.text_font().id()]
            })
            .clone();
        args.cmd.spawn((
            LevelObject,
            self.trans,
            Text3d::new(self.text.clone()),
            Text3dStyling {
                font: font.into(),
                size: self.pt,
                world_scale: Some(Vec2::splat(self.pt / 256.0)),
                align: self.align.to_text_align(),
                ..Default::default()
            },
            Mesh3d::default(),
            MeshMaterial3d(self.material.clone()),
        ));
    }
}

impl SerialAlign {
    pub fn to_text_align(self) -> TextAlign {
        match self {
            SerialAlign::Left => TextAlign::Left,
            SerialAlign::Center => TextAlign::Center,
            SerialAlign::Right => TextAlign::Right,
        }
    }
}

impl SerialButton {
    pub fn bind(
        node: &KdlNode,
        _load_context: &mut LoadContext,
        source: Arc<String>,
    ) -> Result<Self, KdlBindError> {
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
    ) -> Result<Self, KdlBindError> {
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
