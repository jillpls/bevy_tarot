//! Sprite management

use crate::{load_asset, MagicianError, AssetPathMap, TarotAsset, HandleMap, SimpleToString, AssetKey};
use std::default::Default;
use bevy_asset::prelude::*;
use bevy_render::prelude::*;
use bevy_ecs::prelude::*;
use bevy_sprite::prelude::*;
use bevy_transform::prelude::*;
use bevy_log::*;
use bevy_reflect::TypePath;

/// Marker trait for asset keys that are used for sprites.
pub trait SpriteAssetKey : AssetKey {}

/// Handle map for `Handle<Image>`
pub type SpriteHandleMap<K> = HandleMap<K, Image>;

/// Handle map for `Handle<SpriteSheet>`
pub type SpriteSheetHandleMap<K> = HandleMap<K, SpriteSheet>;

/// Handle map for `Handle<TextureAtlasLayout>`
pub type TextureAtlasLayoutHandleMap<K> = HandleMap<K, TextureAtlasLayout>;

/// Alias for `AssetPathMap` - TODO: Remove
pub type SpritePathMap<K> = AssetPathMap<K>;

impl TarotAsset for Image {}

/// Trigger event to add a sprite to an existing entity.
#[derive(Event)]
pub struct AddSpriteToEntity<K : SpriteAssetKey> {
    /// Entity id
    pub entity: Entity,
    /// `SpriteAssetKey` of the sprite.
    pub key: K,
    /// Index of the sprite if its part of a sprite sheet.
    pub index: Option<usize>,
}

/// Triggered system for adding sprites to entities.
pub fn add_sprite_to_entity<K : SpriteAssetKey>(
    trigger: Trigger<AddSpriteToEntity<K>>,
    query: Query<&Transform>,
    mut commands: Commands,
    sprite_handle_map: Res<SpriteHandleMap<K>>,
    sprite_sheet_handle_map: Res<SpriteSheetHandleMap<K>>,
    sprite_sheet_data: Res<Assets<SpriteSheet>>,
    mut atlas_layout_handle_map: ResMut<TextureAtlasLayoutHandleMap<K>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    match try_add_sprite_to_entity(
        trigger.event(),
        query,
        &mut commands,
        &sprite_handle_map,
        &sprite_sheet_handle_map,
        &sprite_sheet_data,
        &mut atlas_layout_handle_map,
        &mut atlas_layouts,
    ) {
        Ok(()) => {}
        Err(e) => {
            warn!("{}", e)
        }
    }
}

/// Try to add sprite to entity (should rarely be called directly)
fn try_add_sprite_to_entity<K : SpriteAssetKey>(
    event: &AddSpriteToEntity<K>,
    query: Query<&Transform>,
    commands: &mut Commands,
    sprite_handle_map: &Res<SpriteHandleMap<K>>,
    sprite_sheet_handle_map: &Res<SpriteSheetHandleMap<K>>,
    sprite_sheet_data: &Res<Assets<SpriteSheet>>,
    atlas_layout_handle_map: &mut ResMut<TextureAtlasLayoutHandleMap<K>>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> Result<(), MagicianError> {
    let sprite = sprite_handle_map
        .get(&event.key)
        .ok_or(MagicianError::AssetNotFound(event.key.sstr()))?;
    let mut entity = commands
        .get_entity(event.entity)
        .ok_or(MagicianError::EntityNotFound(event.entity))?;
    let transform = *query
        .get(entity.id())
        .map_err(|_| MagicianError::EntityNotFound(entity.id()))?;
    let sprite_bundle = SpriteBundle {
        transform,
        texture: sprite,
        ..Default::default()
    };
    entity.insert(sprite_bundle);
    match try_get_layout(
        &event.key,
        atlas_layout_handle_map,
        sprite_sheet_handle_map,
        sprite_sheet_data,
        atlas_layouts,
    ) {
        Ok(layout) => {
            let atlas = TextureAtlas {
                layout,
                index: event.index.unwrap_or_default(),
            };
            entity.insert(atlas);
        }
        Err(_) => {
            if let Some(index) = event.index {
                warn!(
                    "Added Sprite to Entity {} with index {} but no sprite sheet was found for {}",
                    entity.id(),
                    index,
                    event.key.sstr()
                );
            }
        }
    }
    Ok(())
}

/// Try getting the layout for a specified `SpriteAssetKey`
fn try_get_layout<K : SpriteAssetKey>(
    key: &K,
    atlas_layout_handle_map: &mut ResMut<TextureAtlasLayoutHandleMap<K>>,
    sprite_sheet_handle_map: &Res<SpriteSheetHandleMap<K>>,
    sprite_sheets: &Res<Assets<SpriteSheet>>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> Result<Handle<TextureAtlasLayout>, MagicianError> {
    Ok(atlas_layout_handle_map.get(key).unwrap_or({
        let sprite_sheet = sprite_sheet_handle_map
            .get(key)
            .ok_or(MagicianError::AssetNotFound(format!("{:?}", key)))?;
        let sprite_sheet = sprite_sheets
            .get(&sprite_sheet)
            .ok_or(MagicianError::NotLoaded(
                SpriteSheet::short_type_path().to_string(),
                key.sstr(),
            ))?;
        let atlas_layout: TextureAtlasLayout = sprite_sheet.into();
        let handle = atlas_layouts.add(atlas_layout);
        atlas_layout_handle_map.insert(key.clone(), handle.clone());
        handle
    }))
}

/// Load Asset wrapper for `Handle<Image>`
pub fn load_sprite<K : SpriteAssetKey>(
    key: K,
    sprite_paths: &SpritePathMap<K>,
    sprite_handle_map: &mut SpriteHandleMap<K>,
    asset_server: &AssetServer,
) -> Result<Handle<Image>, MagicianError> {
    load_asset(key, sprite_paths, sprite_handle_map, asset_server)
}

pub use sprite_sheet::*;

mod sprite_sheet {
    //! Sprite sheet reprensetation.
    use serde::{Deserialize, Serialize};
    use std::cell::RefCell;
    use bevy_asset::{AssetLoader, LoadContext, ron, AsyncReadExt};
    use bevy_asset::io::Reader;
    use bevy_math::URect;
    use bevy_reflect::TypePath;
    use bevy_sprite::TextureAtlasLayout;
    use thiserror::Error;
    use crate::*;

    /// Load Asset wrapper for `Handle<SpriteSheet>`
    pub fn load_sprite_sheet<K : SpriteAssetKey>(
        key: K,
        sprite_paths: &SpritePathMap<K>,
        sprite_sheet_handle_map: &mut SpriteSheetHandleMap<K>,
        asset_server: &AssetServer,
    ) -> Result<Handle<SpriteSheet>, MagicianError> {
        load_asset(key, sprite_paths, sprite_sheet_handle_map, asset_server)
    }

    /// Sprite sheet representation
    #[derive(Asset, TypePath, Debug, Serialize, Deserialize)]
    pub struct SpriteSheet {
        /// Layout of sprites
        pub(crate) layout: SpriteSheetLayout,
        /// Size of the base sprite.
        pub(crate) size: (u32, u32),
        #[serde(skip)]
        /// Cache sprite size for grid layout.
        pub(crate) grid_sprite_size: RefCell<Option<(u32, u32)>>,
    }

    impl From<&SpriteSheet> for TextureAtlasLayout {
        fn from(value: &SpriteSheet) -> Self {
            let mut result = TextureAtlasLayout::new_empty(value.size.into());
            (0..value.len())
                .filter_map(|i| value.get(i as u32))
                .for_each(|sprite| {
                    result.add_texture(URect::from_corners(sprite.min.into(), sprite.max.into()));
                });
            result
        }
    }

    unsafe impl Sync for SpriteSheet {}
    unsafe impl Send for SpriteSheet {}

    impl SpriteSheet {
        /// Size
        pub fn size(&self) -> (u32, u32) {
            self.size
        }

        /// Sprite count.
        pub fn len(&self) -> usize {
            match &self.layout {
                SpriteSheetLayout::Grid(g) => g.len(),
                SpriteSheetLayout::List(l) => l.len(),
            }
        }

        /// Get Sprite data for index (grid is enumerated as row1, row2 ...)
        pub fn get(&self, index: u32) -> Option<SpriteData> {
            match &self.layout {
                SpriteSheetLayout::Grid(g) => {
                    // TODO: Check this again
                    let row = index / g.cols;
                    if row >= g.rows {
                        return None;
                    }
                    let col = index % g.cols;
                    // TODO: This seems overkill
                    let sprite_size = {
                        let mut spr_size = self.grid_sprite_size.borrow_mut();
                        if let Some(s) = *spr_size {
                            s
                        } else {
                            let sprite_size = (self.size.0 / g.cols, self.size.1 / g.rows);
                            *spr_size = Some(sprite_size);
                            sprite_size
                        }
                    };
                    let x = sprite_size.0 * col;
                    let y = sprite_size.1 * row;
                    SpriteData::new((x, y), (x + sprite_size.0, y + sprite_size.1)).into()
                }
                SpriteSheetLayout::List(list) => list.get(index as usize).copied(),
            }
        }
    }

    impl TarotAsset for SpriteSheet {
        fn file_extension() -> Option<&'static str> {
            Some("ron")
        }
    }

    /// Grid size for a sprite sheet
    #[derive(Asset, TypePath, Debug, Serialize, Deserialize, Clone)]
    pub struct SpriteSheetGrid {
        /// rows (should be larger than 0)
        rows: u32,
        /// columns (should be larger than 0)
        cols: u32,
    }

    impl SpriteSheetGrid {
        /// Calculate sprite count
        pub fn len(&self) -> usize {
            (self.rows * self.cols) as usize
        }
    }

    /// Layout of a sprite sheet
    #[derive(Asset, TypePath, Debug, Serialize, Deserialize, Clone)]
    #[serde(untagged)]
    pub enum SpriteSheetLayout {
        /// Grid (rows, cols)
        Grid(SpriteSheetGrid),
        /// List with positions and sizes
        List(Vec<SpriteData>),
    }

    /// AssetLoader for `SpriteSheet`
    #[derive(Default)]
    pub struct SpriteSheetLoader {}

    /// Loading errors for `SpriteSheetLoader`
    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum SpriteSheetLoadingError {
        /// An [IO](std::io) Error
        #[error("Could not load asset: {0}")]
        Io(#[from] std::io::Error),
        /// A [RON](ron) Error
        #[error("Could not parse RON: {0}")]
        RonSpannedError(#[from] ron::error::SpannedError),
    }

    impl AssetLoader for SpriteSheetLoader {
        type Asset = SpriteSheet;
        type Settings = ();
        type Error = SpriteSheetLoadingError;

        async fn load<'a>(
            &'a self,
            reader: &'a mut Reader<'_>,
            _settings: &'a Self::Settings,
            _load_context: &'a mut LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            ron::de::from_bytes::<SpriteSheet>(&bytes).map_err(|e| e.into())
        }
    }

    pub use sprite_data::*;
    mod sprite_data {
        //! Sprite data for Sprite Sheets
        use bevy_asset::Asset;
        use bevy_math::Vec2;
        use bevy_reflect::TypePath;
        use serde::{Deserialize, Serialize};
        use super::*;

        /// Data of a psrite for Serialization.
        #[derive(Asset, TypePath, Debug, Serialize, Deserialize, Clone, Copy)]
        pub struct SpriteData {
            /// Min position on sheet.
            pub min: (u32, u32),
            /// Max position on sheet.
            pub max: (u32, u32),
        }

        impl SpriteData {
            /// Simpel constructor
            pub fn new(min: (u32, u32), max: (u32, u32)) -> Self {
                Self { min, max }
            }
        }

        impl From<SpriteData> for Vec2 {
            fn from(value: SpriteData) -> Self {
                Vec2::new(
                    (value.max.0 - value.min.0) as f32,
                    (value.max.1 - value.min.1) as f32,
                )
            }
        }
    }
}
