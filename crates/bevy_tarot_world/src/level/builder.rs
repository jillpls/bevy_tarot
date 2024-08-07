use bevy_math::Quat;
use super::*;
use std::io::BufReader;
use std::path::Path;
use bevy_ecs::prelude::*;
use bevy_log::*;
use bevy_tarot_magician::SpriteAssetKey;
use ron;
use avian2d::prelude::*;
use bevy_math::{Rot2, Vec2};
use bevy_tarot_magician::sprite::AddSpriteToEntity;
use smallvec::SmallVec;
use bevy_transform::prelude::*;
use bevy_tarot_hermit::is_default;
use serde::de::DeserializeOwned;

/// TODO: Placeholder
pub trait WorldLayer : PhysicsLayer + Default + Serialize + DeserializeOwned {}

/// Serializable Level object that can be used to load levels.
/// TODO: Generalize to not require avian
#[derive(Serialize, Deserialize, Component)]
pub struct LevelBuilder<L> {
    /// Level name
    pub name: String,
    /// Unique level id
    /// TODO: What do we do when ids are reused?
    pub id: LevelId,
    /// Static level elements.
    /// TODO: Rethink
    #[serde(default = "Vec::new")]
    pub static_elements: Vec<StaticLevelElementBuilder<L>>,
}

impl<L : WorldLayer> LevelBuilder<L> {
    /// Gets sprites from all elements and tries to convert them into `K : SpriteAssetKey`
    pub fn sprite_keys<K : SpriteAssetKey>(&self) -> HashSet<K> {
        self.static_elements
            .iter()
            .filter_map(|builder| (builder.sprite.clone()).try_into().ok() )
            .collect::<_>()
    }

    /// Tries to deserialize a level from a given path.
    /// TODO: Better error handling
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let f = std::fs::File::open(path);
        match f {
            // TODO: Better error handling
            Ok(f) => ron::de::from_reader(BufReader::new(f)).map_err(|e| { warn!("{:?}", e); e}).ok(),
            Err(e) => {
                warn!("{:?}", e);
                None
            }
        }
    }

    /// TODO: Probably gate this behind a feature flag.
    pub fn spawn<K : SpriteAssetKey + Component>(&self, commands: &mut Commands) {
        info!("Spawning Level: \"{}\" ({})", self.name, self.id);
        for (i, element) in self.static_elements.iter().enumerate() {
            let offset = ((i as f32) / (self.static_elements.len() as f32)) * 0.1;
            let _ = element.spawn_element::<K>(commands, offset, self.id);
        }
    }
}

// TODO: Move to hermit
fn de_none<T>() -> Option<T> {
    None
}

/// Static Element
#[derive(Serialize, Deserialize)]
pub struct StaticLevelElementBuilder<L> {
    /// Position relative to level
    pub position: Vec2,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    /// Draw layer. Higher draw layer is painted in front. Elements on the same draw layer are sorted randomly by default.
    pub draw_layer: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    /// Rotation
    pub rotation: Option<Rot2>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    /// Scale
    pub scale: Option<Vec2>,
    #[serde(default = "de_none::<StaticColliderBuilderBundle<L>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Collider data
    pub collider: Option<StaticColliderBuilderBundle<L>>,
    /// Sprite key as a string
    pub sprite: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    /// Sprite index (for Texture atlas)
    pub sprite_index: Option<usize>,
}

impl<L> StaticLevelElementBuilder<L> {
    /// Simple constructor
    pub fn new<K: AssetKey>(key: K) -> Self {
        Self {
            position: Default::default(),
            draw_layer: 0,
            rotation: None,
            scale: None,
            collider: None,
            sprite: key.into(),
            sprite_index: None,
        }
    }

    /// Add an index to the sprite (TextureAtlas)
    pub fn with_sprite_index(mut self, index: usize) -> Self {
        self.sprite_index = Some(index);
        self
    }

    /// Set Transform
    /// TODO: Rotation
    pub fn set_transform(&mut self, transform: &Transform) {
        self.position = transform.translation.truncate();
        // TODO: Rotation
        if transform.scale.truncate() != Vec2::ONE {
            self.scale = Some(transform.scale.truncate());
        }
    }


    /// Transform with z `offset`
    pub fn layered_transform(&self, offset: f32) -> Transform {
        let mut transform =
            Transform::from_translation(self.position.extend(self.draw_layer as f32 + offset));
        if let Some(r) = self.rotation {
            transform.rotation = Quat::from_rotation_z(r.as_radians());
        }
        if let Some(s) = self.scale {
            transform.scale = s.extend(0.);
        }
        transform
    }

    /// Base Transform
    pub fn transform(&self) -> Transform {
        self.layered_transform(0.)
    }
}

impl<L : WorldLayer> StaticLevelElementBuilder<L> {
    /// TODO: Probably gate this behind a feature flag.
    pub fn spawn_element<K : SpriteAssetKey + Component>(&self, commands: &mut Commands, offset: f32, id: LevelId) -> Result<Entity, ()> {
        let transform = self.layered_transform(offset);
        let key: K = self.sprite.clone().try_into().map_err(|e| ())?; // TODO
        let mut entity = commands.spawn((transform, key.clone(), id));
        if let Some(c) = &self.collider {
            entity.insert(c.collider.build());
            entity.insert(c.layers.build());
            if c.sensor {
                entity.insert(Sensor);
            } else {
                entity.insert(RigidBody::Static);
            }
        }
        let id = entity.id();
        commands.trigger(AddSpriteToEntity {
            entity: id,
            key,
            index: self.sprite_index,
        });
        Ok(id)
    }
}

/// TODO: PLACEHOLDER
#[derive(Serialize, Deserialize)]
pub struct StaticColliderBuilderBundle<L> {
    /// TODO: PLACEHOLDER
    pub collider: StaticCollider,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    /// TODO: PLACEHOLDER
    pub sensor: bool,
    /// TODO: PLACEHOLDER
    pub layers: CollisionLayerBuilder<L>,
}

impl<L : PartialEq> PartialEq for StaticColliderBuilderBundle<L> {
    fn eq(&self, other: &Self) -> bool {
        self.sensor == other.sensor && self.layers.eq(&other.layers)
    }
}

/// TODO:: PLACEHOLDER
#[derive(Serialize, Deserialize)]
pub enum CollisionLayerBuilder<L> {
    /// TODO:: PLACEHOLDER
    Avian2d(CollisionLayers),
    /// TODO:: PLACEHOLDER
    Lists(SmallVec<[L; 32]>, SmallVec<[L; 32]>),
}

impl<L : WorldLayer> CollisionLayerBuilder<L> {
    /// TODO:: PLACEHOLDER
    pub fn build(&self) -> CollisionLayers {
        match self {
            CollisionLayerBuilder::Avian2d(l) => *l,
            CollisionLayerBuilder::Lists(m, f) => {
                let mut members = LayerMask::NONE;
                let mut filters = LayerMask::NONE;
                for member in m {
                    members.add(member);
                }
                for filter in f {
                    filters.add(filter)
                }
                CollisionLayers::new(members, filters)
            }
        }
    }
}

impl<L : PartialEq> PartialEq for CollisionLayerBuilder<L> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Avian2d(_), Self::Lists(_,_)) | (Self::Lists(_,_), Self::Avian2d(_)) => { false }
            (Self::Avian2d(first), Self::Avian2d(second)) => first == second,
            (Self::Lists(m1, f1), Self::Lists(m2, f2)) => {
                m1 == m2 && f1 == f2}
            }
        }
    }

impl<L> Default for CollisionLayerBuilder<L> {
    fn default() -> Self {
        // TODO: is this a good default?
        Self::Avian2d(CollisionLayers::new(LayerMask::ALL, LayerMask::ALL))
    }
}

#[derive(Serialize, Deserialize)]
/// TODO:: PLACEHOLDER
pub enum StaticCollider {
    /// TODO:: PLACEHOLDER
    Avian2d(Collider),
}

/// TODO:: PLACEHOLDER
impl StaticCollider {
    /// TODO:: PLACEHOLDER
    pub fn build(&self) -> Collider {
        match self {
            StaticCollider::Avian2d(c) => c.clone(),
        }
    }
}
