use bevy_state::prelude::States;

#[derive(States, Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub enum TemperanceState {
    LevelEditor,
    #[default]
    SpriteSheetEditor,
}