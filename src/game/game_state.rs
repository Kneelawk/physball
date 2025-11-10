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
                PreUpdate,
                pause_play.run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            )
            .add_systems(Update, show_hide_cursor);

        #[cfg(target_arch = "wasm32")]
        {
            wasm::wasm_build(app);
        }
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        if input.just_pressed(Key::Escape) {
            next_state.set(!*state.get());
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if input.just_pressed(Key::Character("`".into())) {
            next_state.set(!*state.get());
        }
    }
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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::game::game_state::GameState;
    use crate::or_return;
    use bevy::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::wasm_bindgen::closure::Closure;

    pub fn wasm_build(app: &mut App) {
        app.init_resource::<WasmEscapeListener>()
            .add_systems(Startup, add_wasm_escape_listener)
            .add_systems(
                PreUpdate,
                wasm_pause_play
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            );
    }

    #[derive(Debug, Default, Clone, Resource, Reflect)]
    #[reflect(Debug, Default, Clone, Resource)]
    pub struct WasmEscapeListener {
        // wasm is single-threaded, but no sense in not future-proofing it when it's this easy
        pub pointer_lock_change: Arc<AtomicU32>,
    }

    fn add_wasm_escape_listener(res: Res<WasmEscapeListener>) {
        let pointer_lock_change = res.pointer_lock_change.clone();

        let window = or_return!(Option(web_sys::window()));
        let document = or_return!(Option(window.document()));

        let cb = Closure::new(Box::new(move || {
            let window = or_return!(Option(web_sys::window()));
            let document = or_return!(Option(window.document()));
            let id = if let Some(element) = document.pointer_lock_element()
                && element.tag_name() == "CANVAS"
            {
                1
            } else {
                2
            };

            pointer_lock_change.store(id, Ordering::Release);
        }) as Box<dyn FnMut()>);

        document
            .add_event_listener_with_callback("pointerlockchange", &cb.as_ref().unchecked_ref())
            .unwrap();

        cb.forget();
    }

    fn wasm_pause_play(mut next_state: ResMut<NextState<GameState>>, res: Res<WasmEscapeListener>) {
        if res.pointer_lock_change.swap(0, Ordering::AcqRel) == 2 {
            next_state.set(GameState::Paused);
        }
    }
}
