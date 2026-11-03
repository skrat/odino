use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Init,
    Running,
    Paused,
}

#[derive(Resource)]
pub struct Speed(pub f32);

#[derive(Resource, Deref, DerefMut)]
pub struct Score(pub f32);

pub fn not_in_state<S: States>(state: S) -> impl FnMut(Option<Res<State<S>>>) -> bool + Clone {
    move |current_state: Option<Res<State<S>>>| match current_state {
        Some(current_state) => *current_state != state,
        None => true,
    }
}
