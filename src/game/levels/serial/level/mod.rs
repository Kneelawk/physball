use crate::capture_result;
use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::levels::finish_point::FinishPoint;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::KdlDocumentExt;
use crate::game::levels::serial::level::button::{SerialButton, SerialButtonDoor};
use crate::game::levels::serial::level::cuboid::SerialCuboid;
use crate::game::levels::serial::level::dynamic::SerialDynamicObject;
use crate::game::levels::serial::level::music::{SerialMusic, SerialTriggeredMusic};
use crate::game::levels::serial::level::plane::SerialPlane;
use crate::game::levels::serial::level::text::SerialText;
use crate::game::levels::{LevelObject, PlayerSpawnPoint};
use bevy::asset::LoadContext;
use bevy::prelude::*;
use kdl::KdlDocument;
use std::collections::HashMap;
use std::sync::Arc;

mod button;
mod cuboid;
mod dynamic;
mod music;
mod plane;
mod text;

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
    pub cuboids: Vec<SerialCuboid>,
    pub texts: Vec<SerialText>,
    pub buttons: Vec<SerialButton>,
    pub button_doors: Vec<SerialButtonDoor>,
    pub dynamic_objects: Vec<SerialDynamicObject>,
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

        let cuboids = doc
            .nodes()
            .iter()
            .filter(|node| node.name().value() == "cuboid")
            .map(|node| SerialCuboid::bind(node, load_context, source.clone()))
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
            cuboids,
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
            cuboids,
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
            cuboids,
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

        for cuboid in self.cuboids.iter() {
            cuboid.spawn(args);
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
