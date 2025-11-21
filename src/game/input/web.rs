use super::{PlayerInput, desktop};
use crate::game::CANVAS_ID;
use crate::game::game_state::GameState;
use crate::or_return;
use bevy::input::keyboard::Key;
use bevy::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::closure::Closure;

pub fn build(app: &mut App) {
    app.init_resource::<WasmEscapeListener>()
        .add_systems(Startup, add_escape_listener)
        .add_systems(
            PreUpdate,
            (
                pause_on_lose_focus,
                pause_play,
                desktop::debug_gizmos,
                desktop::keyboard_input,
                desktop::mouse_input,
                desktop::mouse_scroll,
            ),
        );
}

pub fn game_started(state: &mut NextState<GameState>) {
    let window = or_return!(_r => {
            state.set(GameState::Paused);
            return;
        } : Option(web_sys::window()));
    let document = or_return!(_r => {
            state.set(GameState::Paused);
            return;
        } : Option(window.document()));
    let active_element = or_return!(_r => {
            state.set(GameState::Paused);
            return;
        } : Option(document.active_element()));

    if active_element.id() == CANVAS_ID {
        state.set(GameState::Playing);
    } else {
        state.set(GameState::Paused);
    }
}

pub fn pause_play(mut writer: MessageWriter<PlayerInput>, input: Res<ButtonInput<Key>>) {
    if input.just_pressed(Key::Escape) {
        // in case the game thinks it grabbed the cursor but didn't actually
        writer.write(PlayerInput::Pause { toggle: false });
    }
    if input.just_pressed(Key::Character("`".into())) {
        writer.write(PlayerInput::Pause { toggle: true });
    }
}

#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Debug, Default, Clone, Resource)]
pub struct WasmEscapeListener {
    // wasm is single-threaded, but no sense in not future-proofing it when it's this easy
    pub pointer_lock_change: Arc<AtomicU32>,
}

fn add_escape_listener(res: Res<WasmEscapeListener>) {
    let pointer_lock_change = res.pointer_lock_change.clone();

    let window = or_return!(Option(web_sys::window()));
    let document = or_return!(Option(window.document()));

    let cb = Closure::new(Box::new(move || {
        let window = or_return!(Option(web_sys::window()));
        let document = or_return!(Option(window.document()));
        let id = if let Some(element) = document.pointer_lock_element()
            && element.id() == CANVAS_ID
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

fn pause_on_lose_focus(mut writer: MessageWriter<PlayerInput>, res: Res<WasmEscapeListener>) {
    if res.pointer_lock_change.swap(0, Ordering::AcqRel) == 2 {
        writer.write(PlayerInput::Pause { toggle: false });
    }
}
