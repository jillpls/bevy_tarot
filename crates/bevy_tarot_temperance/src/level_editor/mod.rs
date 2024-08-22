use bevy_app::App;
use bevy_ecs::component::Component;
use bevy_state::prelude::States;
use crate::AssetKey;

pub fn plugin<K : AssetKey + Component, S : States + Copy>(app: &mut App, state: S) {

}
