#![warn(missing_docs)]
//! Asset management

use serde::{Deserialize, Serialize};
use sprite::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use thiserror::Error;
use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::*;
use bevy_tarot_hermit::{HermitError, SimpleToString};

mod animation;
pub mod sprite;
pub use sprite::SpriteAssetKey; // TODO: Prelude
pub use bevy_asset::AssetServer;

/// Assets plugin
pub fn plugin<K : SpriteAssetKey>(app: &mut App) {
    app.init_asset::<SpriteSheet>();
    app.init_asset_loader::<SpriteSheetLoader>();
    app.insert_resource(SpriteHandleMap::<K>::default());
    app.insert_resource(SpriteSheetHandleMap::<K>::default());
    app.insert_resource(TextureAtlasLayoutHandleMap::<K>::default());
    app.insert_resource(SpritePathMap::<K>::default());
    app.observe(add_sprite_to_entity::<K>);
}

/// Errors created by the Magician Crate.
#[derive(Error, Debug)]
pub enum MagicianError {
    /// Asset could not be loaded (path, key)
    #[error("Asset not Loaded: {0:?} [{1:?}]")]
    NotLoaded(String, String),
    /// Asset was not found but expected to be loaded.
    #[error("Could not find sprite handle for {0:?}")]
    AssetNotFound(String),
    /// Entity was not found.
    #[error("Entity {0:?} not found.")]
    EntityNotFound(Entity),
    /// Generic error
    #[error("<Hermit Error> {0}")]
    HermitError(HermitError),
}

/// Trait to mark Assets in this crate.
pub trait TarotAsset: Asset + Debug {
    /// Assets have an associated file extension.
    /// TODO: it would be nice to also differentiate between {name}.ron and {name}_anim.ron
    fn file_extension() -> Option<&'static str> {
        None
    }
}

/// Load assets and discard errors.
pub fn load_assets_unchecked<K: AssetKey, T: TarotAsset>(
    keys: Vec<K>,
    paths: &AssetPathMap<K>,
    handle_map: &mut HandleMap<K, T>,
    asset_server: &AssetServer,
    print_warnings: bool,
) -> Vec<Handle<T>> {
    load_assets(keys, paths, handle_map, asset_server)
        .into_iter()
        .filter_map(|r| {
            r.map_err(|e| {
                if print_warnings {
                    warn!("{:?}", &e);
                }
                e
            })
                .ok()
        })
        .collect::<_>()
}

/// Load assets
pub fn load_assets<K: AssetKey, T: TarotAsset>(
    keys: Vec<K>,
    paths: &AssetPathMap<K>,
    handle_map: &mut HandleMap<K, T>,
    asset_server: &AssetServer,
) -> Vec<Result<Handle<T>, MagicianError>> {
    keys.into_iter()
        .map(|k| load_asset(k, paths, handle_map, asset_server))
        .collect::<_>()
}

/// Load asset
pub fn load_asset<K: AssetKey, T: TarotAsset>(
    key: K,
    paths: &AssetPathMap<K>,
    handle_map: &mut HandleMap<K, T>,
    asset_server: &AssetServer,
) -> Result<Handle<T>, MagicianError> {
    let path = paths
        .get(&key)
        .map(|p| {
            if let Some(file_ext) = T::file_extension() {
                let mut path = PathBuf::from(p);
                path.set_extension(file_ext);
                path
            } else {
                PathBuf::from(p)
            }
        })
        .ok_or(MagicianError::AssetNotFound(format!(
            "{:?} [No path saved]",
            key
        )))?;
    {
        // TODO: Seems expensive ...
        let mut p = PathBuf::from("assets");
        p.push(path.as_path());
        if !p.exists() {
            return Err(MagicianError::AssetNotFound(format!("{:?}", key)));
        }
    }
    let handle: Handle<T> = asset_server.load(path);
    handle_map.insert(key, handle.clone());
    Ok(handle)
}

/// AssetKey
pub trait AssetKey: Sized + Clone + Hash + Eq + Debug + Send + Sync + TryFrom<String> + Into<String> + 'static {
    /// If the path is stored inside the asset key return it.
    fn path(&self) -> Option<&String> {
        None
    }
}

/// Map that stores paths for specified asset keys.
#[derive(Resource, Serialize, Deserialize, Debug)]
pub struct AssetPathMap<T: AssetKey>(HashMap<T, String>);

impl<T: AssetKey> Default for AssetPathMap<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T: AssetKey> AssetPathMap<T> {
    /// Get path from map or from asset key if it stores the path itself.
    /// Paths stored by the `AssetKey` take precedence
    pub fn get<'a>(&'a self, key: &'a T) -> Option<&'a String> {
        if let Some(p) = key.path() {
            Some(p)
        } else {
            self.0.get(key)
        }
    }
}

/// Map that stores Handles for `AssetKey`s
#[derive(Resource, Debug)]
pub struct HandleMap<K: AssetKey, A: Asset> {
    /// List of Handles
    handles: Vec<Handle<A>>,
    /// Map from `AssetKey` to index of `handles`
    map: HashMap<K, usize>,
    /// Map from (loading/loaded) `AssetId` to `AssetKey`
    id_to_key: HashMap<AssetId<A>, K>,
}

impl<K: AssetKey, A: Asset> Default for HandleMap<K, A> {
    fn default() -> Self {
        Self {
            handles: vec![],
            map: HashMap::new(),
            id_to_key: HashMap::new(),
        }
    }
}

impl<K: AssetKey + Hash + Eq, A: Asset> HandleMap<K, A> {

    /// `AssetKey`->index map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get associated `AssetKey` for a given `AssetId`
    pub fn get_key(&self, id: &AssetId<A>) -> Option<&K> {
        self.id_to_key.get(id)
    }

    /// Directly get handle.
    pub fn get(&self, key: &K) -> Option<Handle<A>> {
        self.map
            .get(key)
            .and_then(|i| self.handles.get(*i).cloned())
    }

    /// Insert new key, handle and id.
    pub fn insert(&mut self, key: K, handle: Handle<A>) {
        if self.map.contains_key(&key) {
            return;
        }
        self.id_to_key.insert(handle.id(), key.clone());
        self.map.insert(key, self.handles.len());
        self.handles.push(handle);
    }
}

impl<K: AssetKey, A: Asset> HandleMap<K, A> {
    /// Check if all assets in the handle map are loaded.
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.map
            .values()
            .filter_map(|i| self.handles.get(*i))
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }
}

/// TODO: Whats up with this?
pub fn get_associated_file<K: AssetKey>(
    key: &K,
    sprite_paths: &AssetPathMap<K>,
    file_ending: &str,
) -> Option<PathBuf> {
    sprite_paths.get(key).and_then(|p| {
        let path = Path::new(p);
        if path.exists() {
            let mut path_buf = path.to_path_buf();
            path_buf.set_extension(file_ending);
            Some(path_buf)
        } else {
            None
        }
    })
}
