use crate::game::state::AppState;
use avian3d::prelude::{Physics, PhysicsTime};
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use std::ops::Not;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(AppState::Game), game_started)
            .add_systems(OnExit(AppState::Game), game_stopped)
            .add_systems(
                Update,
                pause_play.run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            )
            .add_systems(Update, show_hide_cursor);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, States, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Disabled,
    Playing,
    Paused,
    Finished,
}

impl Not for GameState {
    type Output = GameState;

    fn not(self) -> Self::Output {
        match self {
            GameState::Disabled => GameState::Disabled,
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::Finished => GameState::Finished,
        }
    }
}

fn game_started(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}

fn game_stopped(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Disabled);
}

fn pause_play(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<Key>>,
) {
    if input.just_pressed(Key::Escape) {
        next_state.set(!*state.get());
    }
    // #[cfg(target_arch = "wasm32")]
    // {
    //     if input.just_pressed(Key::Character("`".into())) {
    //         next_state.set(!*state.get());
    //     }
    // }
}

fn show_hide_cursor(
    cur_state: Res<State<GameState>>,
    mut cursor: Single<&mut CursorOptions>,
    mut physics: ResMut<Time<Physics>>,
) {
    if cur_state.is_changed() {
        if *cur_state == GameState::Playing {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
            physics.unpause();
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
            physics.pause();
        }
    }
}
