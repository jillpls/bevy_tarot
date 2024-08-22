use bevy::prelude::*;
use bevy_tarot_temperance::{TemperancePlugin, AssetKey};
use bevy_tarot_temperance::sheet_edit::LoadSprite;

#[derive(States, Default, Debug, Hash, Copy, Clone, Eq, PartialEq)]
pub enum State {
    #[default]
    Editor
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Component)]
pub struct SimpleAssetKey {
    path: String
}

impl TryFrom<String> for SimpleAssetKey {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self { path: value })
    }
}

impl Into<String> for SimpleAssetKey {
    fn into(self) -> String {
        self.path
    }
}

impl AssetKey for SimpleAssetKey {}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.init_state::<State>();
    app.enable_state_scoped_entities::<State>();
    app.add_plugins(TemperancePlugin::<State, SimpleAssetKey>::default());
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Startup, spawn_example);
    app.add_plugins(bevy_tarot_magician::plugin::<SimpleAssetKey>);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_example(mut commands: Commands) {
    commands.trigger(LoadSprite { path: "example.png".to_string() })
}