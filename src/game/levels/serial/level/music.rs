use crate::game::levels::LevelObject;
use crate::game::levels::serial::error::{KdlBindError, MergeKdlBindError};
use crate::game::levels::serial::kdl_utils::{KdlDocumentExt, KdlNodeExt};
use crate::game::levels::serial::level::LevelBuildArgs;
use crate::game::music::{BackgroundMusic, BackgroundMusicTrigger, PLAYBACK_SETTINGS};
use avian3d::prelude::*;
use bevy::asset::LoadContext;
use bevy::prelude::*;
use kdl::KdlNode;
use std::sync::Arc;

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
            PLAYBACK_SETTINGS,
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

        let dimensions = node.must_get_scale(1, &source);

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
