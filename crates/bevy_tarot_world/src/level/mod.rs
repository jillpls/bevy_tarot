//! Levels are areas that are loaded together

pub mod builder;
pub use builder::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use bevy_ecs::prelude::*;
use bevy_math::{Rect, Vec2};
use bevy_tarot_hermit::math::dist_to_rect;
use bevy_tarot_magician::sprite::{load_sprite, load_sprite_sheet, SpriteHandleMap, SpritePathMap, SpriteSheetHandleMap};
use bevy_tarot_magician::{AssetKey, AssetServer, SpriteAssetKey};
use ron::de::SpannedError;

/// Start loading all assets in a level.
pub fn load_level_assets<K : SpriteAssetKey, L : WorldLayer>(
    level: &LevelBuilder<L>,
    asset_server: &AssetServer,
    sprite_paths: &SpritePathMap<K>,
    sprite_handle_map: &mut SpriteHandleMap<K>,
    sprite_sheet_handle_map: &mut SpriteSheetHandleMap<K>,
) {
    for sprite in level.sprite_keys::<K>() {
        let _ = load_sprite(
            sprite.clone(),
            sprite_paths,
            sprite_handle_map,
            asset_server,
        );
        let _ = load_sprite_sheet(
            sprite.clone(),
            sprite_paths,
            sprite_sheet_handle_map,
            asset_server,
        );
    }
}

/// Struct that holds data about all current levels
#[derive(Component, Resource, Default)]
pub struct LevelReference {
    /// Lookup for loading levels
    pub lookup: LevelLoaderLookup,
    /// Levels that are currently loading
    pub loading: HashSet<LevelId>,
    /// Loaded levels
    pub loaded: HashSet<LevelId>,
}

impl LevelReference {
    /// Check if a level is either loaded or currently loading.
    pub fn is_loading_or_loaded(&self, id: &LevelId) -> bool {
        self.loading.contains(id) || self.loaded.contains(id)
    }
    /// Set a level to loaded.
    pub fn set_loaded(&mut self, id: LevelId) {
        self.loading.remove(&id);
        self.loaded.insert(id);
    }
}

/// Level that knows which levels are adjacent
#[derive(Component, Debug, Clone)]
pub struct Level {
    /// Unique id
    pub id: LevelId,
    /// Adjacent levels (these will be preloaded while in the level)
    pub adjacent_levels: HashSet<LevelId>,
}

/// Get the Entity for a level (that is usually stored in LevelReference)
#[derive(Default)]
pub struct LevelLookup {
    pub map: HashMap<LevelId, Entity>,
}

/// Stores paths to Level .ron files
#[derive(Default)]
pub struct LevelLoaderLookup {
    pub map: HashMap<LevelId, String>,
}

/// Unique Id for levels.
#[derive(Serialize, Deserialize, Copy, Clone, Component, Hash, PartialEq, Eq, Debug)]
pub struct LevelId(pub usize);

impl TryFrom<String> for LevelId {
    type Error = SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::de::from_str(&value)
    }
}

impl Into<String> for LevelId {
    fn into(self) -> String {
        self.to_string()
    }
}

impl AssetKey for LevelId {}

impl std::fmt::Display for LevelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
