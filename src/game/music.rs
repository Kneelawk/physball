use crate::game::levels::LevelObject;
use crate::game::logic::Player;
use avian3d::prelude::*;
use bevy::prelude::*;

pub const PLAYBACK_SETTINGS: PlaybackSettings = PlaybackSettings::LOOP;

#[derive(Debug, Default)]
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_trigger_collision);
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash)]
pub struct BackgroundMusic(pub AssetId<AudioSource>);

// TODO: this should probably be script driven
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash)]
#[require(CollisionEventsEnabled)]
pub struct BackgroundMusicTrigger(pub Handle<AudioSource>);

fn on_trigger_collision(
    collision: On<CollisionStart>,
    mut cmd: Commands,
    trigger: Query<(&BackgroundMusicTrigger, Has<LevelObject>)>,
    player: Query<(), With<Player>>,
    background_music: Query<(Entity, &BackgroundMusic)>,
) {
    let Ok((trigger, is_level_object)) = trigger.get(collision.collider1) else {
        return;
    };

    if !player.contains(collision.collider2) {
        return;
    }

    let mut should_spawn = true;
    for (music_entity, music) in background_music {
        if music.0 == trigger.0.id() {
            should_spawn = false;
        } else {
            cmd.entity(music_entity).despawn();
        }
    }

    if should_spawn {
        if is_level_object {
            cmd.spawn((
                LevelObject,
                BackgroundMusic(trigger.0.id()),
                PLAYBACK_SETTINGS,
                AudioPlayer(trigger.0.clone()),
            ));
        } else {
            cmd.spawn((
                BackgroundMusic(trigger.0.id()),
                PLAYBACK_SETTINGS,
                AudioPlayer(trigger.0.clone()),
            ));
        }
    }
}
