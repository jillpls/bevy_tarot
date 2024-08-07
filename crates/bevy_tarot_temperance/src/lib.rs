#![warn(missing_docs)]
//! Level editor

mod input;
mod ui;

use crate::input::EditorAction;
use avian2d::prelude::{Collider, CollidingEntities};
use bevy_app::{App, Update};
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_egui::EguiUserTextures;
use bevy_math::prelude::*;
use bevy_state::state::{OnExit, States};
use bevy_tarot_chariot::keyboard::KeyCode;
use bevy_tarot_chariot::{ButtonInput, ButtonMapping};
use bevy_tarot_hermit::unwrap_option_continue;
use bevy_tarot_hermit::*;
use bevy_tarot_world::level::{LevelBuilder, LevelElement};
use bevy_tarot_world::magician::bevy_asset::{AssetEvent, AssetServer, Assets, Handle, Asset};
use bevy_tarot_world::magician::bevy_render::prelude::Camera;
use bevy_tarot_world::magician::bevy_sprite::{
    Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout,
};
use bevy_tarot_world::magician::sprite::{
    load_sprite, load_sprite_sheet, SpriteHandleMap, SpritePathMap, SpriteSheet,
    SpriteSheetHandleMap,
};
use bevy_tarot_world::magician::AssetKey;
use bevy_transform::prelude::{GlobalTransform, Transform};
use bevy_window::{PrimaryWindow, Window};
use std::ops::{Index, IndexMut};
use bevy_state::prelude::{in_state, OnEnter};

/// TODO: Remove again
pub const SNAP_SIZE: f32 = 24.;

pub struct TemperancePlugin<S: States + Copy, K : AssetKey> {
    state: S,
    _asset_key_dummy: Option<K>
}

impl<S : States + Copy, K : AssetKey> TemperancePlugin<S, K> {
    pub fn new(state: S) -> Self {
        Self {
            state,
            _asset_key_dummy : None
        }
    }
}

impl<S : States + Copy + Default, K : AssetKey + Component> Default for TemperancePlugin<S, K> {
    fn default() -> Self {
        Self {
            state : S::default(),
            _asset_key_dummy: None
        }
    }
}

impl<S: States + Copy, K : AssetKey + Component> bevy_app::Plugin for TemperancePlugin<S, K> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.state), editor_load_textures::<K>);
        app.add_systems(Update, editor_add_sprite::<K>.run_if(in_state(self.state)));

        app.add_systems(OnEnter(self.state), on_enter);
        app.add_systems(OnExit(self.state), on_exit::<K>);

        app.add_systems(Update, update_preview_object_collision_warning);
        app.add_systems(
            Update,
            update_editor_preview_object_pos.run_if(in_state(self.state)),
        );
        app.observe(spawn_editor_preview_object::<K>);
        app.observe(deselect);
        app.observe(place_selection::<K>);

        input::plugin(app, self.state);
        ui::plugin::<S, K>(app, self.state);
    }
}

pub fn on_enter(mut commands: Commands) {}

pub fn on_exit<K: AssetKey>(mut commands: Commands) {
    commands.remove_resource::<SelectableSprites<K>>();
}


#[derive(Component)]
pub struct SelectedEditorObjectPreview {
    pub colliding: bool,
}

#[derive(Event)]
pub struct SetSelectedEditorObject<K: AssetKey> {
    pub key: K,
    pub atlas: TextureAtlas,
}

#[derive(Event)]
pub struct EditorDeselect {}

pub fn deselect(
    _trigger: Trigger<EditorDeselect>,
    mut commands: Commands,
    obj: Query<Entity, With<SelectedEditorObjectPreview>>,
) {
    let e = get_single!(obj);
    unwrap_option!(commands.get_entity(e)).despawn();
}

/// Translates the cursor position to a world position.
/// * `window` - Window that contains the cursor.
/// * `camera` - Camera view.
/// * `camera_transform` - Transform of the camera view.
/// TODO: Utility function should be somewhere else
pub fn cursor_to_world_pos(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world(camera_transform, cursor)
            .map(|ray| ray.origin.truncate())
    })
}

/// Lower left corner of the sprite (for aligning with the grid)
/// TODO: Should we make this a setting to save the final position as lower left as well?
#[derive(Component, Copy, Clone, Debug)]
pub struct LowerLeft(Vec2);

pub fn update_editor_preview_object_pos(
    mut obj: Query<(&LowerLeft, &mut Transform), With<SelectedEditorObjectPreview>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>, // TODO: Make sure its the primary camera
) {
    if let Ok((lower_left, mut obj_transform)) = obj.get_single_mut() {
        let window = window.single();
        let (camera, camera_transform) = camera.single();
        if let Some(pos) = cursor_to_world_pos(window, camera, camera_transform) {
            let mut pos = pos + lower_left.0 - Vec2::new(SNAP_SIZE / 2., SNAP_SIZE / 2.);
            pos.x = (pos.x / SNAP_SIZE).ceil() * SNAP_SIZE;
            pos.y = (pos.y / SNAP_SIZE).ceil() * SNAP_SIZE;
            pos -= lower_left.0;
            obj_transform.translation.x = pos.x;
            obj_transform.translation.y = pos.y;
        }
    }
}

pub fn update_preview_object_collision_warning(
    mut query: Query<(
        &CollidingEntities,
        &mut SelectedEditorObjectPreview,
        &mut bevy_tarot_world::magician::bevy_sprite::Sprite,
    )>,
) {
    let (colliding, mut obj, mut sprite) = get_single_mut!(query);
    if colliding.0.is_empty() == obj.colliding {
        // Change
        obj.colliding = !obj.colliding;
        if obj.colliding {
            sprite.color = Color::srgba(1., 0., 0., 1.);
        } else {
            sprite.color = Color::WHITE
        }
    }
}

use colliders::*;
mod colliders {
    use std::ops::{Index, IndexMut};
    use avian2d::prelude::Collider;
    use bevy_tarot_world::magician::sprite::{SpriteData, SpriteSheet};

    fn collider_from_sprite_data(data: &SpriteData) -> Collider {
        Collider::rectangle(
            (data.max.0 - data.min.0) as f32 - 0.25,
            (data.max.1 - data.min.1) as f32 - 0.25,
        )
    }

    pub struct SpriteSheetColliders {
        colliders: Vec<Collider>,
    }

    impl Index<usize> for SpriteSheetColliders {
        type Output = Collider;

        fn index(&self, index: usize) -> &Self::Output {
            &self.colliders[index]
        }
    }

    impl IndexMut<usize> for SpriteSheetColliders {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.colliders[index]
        }
    }

    impl From<&SpriteSheet> for SpriteSheetColliders {
        fn from(value: &SpriteSheet) -> Self {
            let colliders = (0..value.len())
                .filter_map(|i| value.get(i as u32))
                .map(|sd| collider_from_sprite_data(&sd))
                .collect::<Vec<Collider>>();
            Self { colliders }
        }
    }

    impl From<SpriteSheet> for SpriteSheetColliders {
        fn from(value: SpriteSheet) -> Self {
            (&value).into()
        }
    }
}

pub fn spawn_editor_preview_object<K: AssetKey + Component>(
    trigger: Trigger<SetSelectedEditorObject<K>>,
    mut current: Query<(Entity, &Transform), With<SelectedEditorObjectPreview>>,
    mut commands: Commands,
    sprite_handle_map: Res<SpriteHandleMap<K>>,
    sprite_sheet_handle_map: Res<SpriteSheetHandleMap<K>>,
    sprite_sheet_data_assets: Res<Assets<SpriteSheet>>,
) {
    let atlas = &trigger.event().atlas;
    let index = atlas.index;
    let sprite = unwrap_option!(sprite_handle_map.get(&trigger.event().key));
    let sprite_sheet = unwrap_option!(sprite_sheet_handle_map
        .get(&trigger.event().key)
        .and_then(|sheet| sprite_sheet_data_assets.get(&sheet)));

    let collider_lookup: SpriteSheetColliders = sprite_sheet.into();
    let collider = collider_lookup[index].clone();
    let to_center: Vec2 = collider.shape().as_cuboid().unwrap().half_extents.into();

    let transform = if let Ok((ent, transform)) = current.get_single() {
        let t = *transform;
        commands.get_entity(ent).unwrap().despawn();
        t
    } else {
        Transform::default()
    };

    let sprite_bundle = SpriteBundle {
        transform,
        texture: sprite,
        ..Default::default()
    };

    commands.spawn((
        sprite_bundle,
        atlas.clone(),
        SelectedEditorObjectPreview { colliding: false },
        collider,
        // StateScoped(Screen::Editor), TODO: save the state somewhere so we can reenable this
        LowerLeft(-to_center),
        trigger.event().key.clone(),
    ));
}

fn edge_dist(pos: f32, size: f32) -> f32 {
    if pos > size / 2. {
        size - pos
    } else {
        pos
    }
}

fn get_pan_speed(pos: f32, size: f32, max_edge_dist: f32) -> f32 {
    let edge_dist = edge_dist(pos, size);
    if edge_dist < max_edge_dist {
        max_edge_dist - edge_dist
    } else {
        0.
    }
}

fn get_pan_speed_signed(pos: f32, size: f32, max_edge_dist: f32) -> f32 {
    let pan_speed = get_pan_speed(pos, size, max_edge_dist);
    if pos > size / 2. {
        pan_speed
    } else {
        -pan_speed
    }
}

/// TODO: Shouldnt this be in input?
fn update_move_axis(
    move_axis: &mut f32,
    positive_action: EditorAction,
    negative_action: EditorAction,
    mapping: &ButtonMapping<EditorAction>,
    input: &ButtonInput<KeyCode>,
) {
    if *move_axis == 0. {
        if mapping.pressed(&positive_action, Some(input), None, None) {
            *move_axis += 10.;
        }

        if mapping.pressed(&negative_action, Some(input), None, None) {
            *move_axis -= 10.;
        }
    }
}

#[derive(Event)]
pub struct EditorPlace {}

#[derive(Component)]
pub struct PlacedObject {}

pub fn place_selection<K: AssetKey + Component>(
    _trigger: Trigger<EditorPlace>,
    mut commands: Commands,
    mut selection: Query<
        (
            Entity,
            &mut Transform,
            Option<&CollidingEntities>,
            &mut Sprite,
            &TextureAtlas,
            &K,
        ),
        With<SelectedEditorObjectPreview>,
    >,
) {
    let (entity, mut transform, colliding, mut sprite, atlas, key) = get_single_mut!(selection);
    let atlas = atlas.clone();
    if colliding.map(|c| !c.0.is_empty()).unwrap_or_default() {
        return;
    }
    transform.translation.z -= 100.;
    sprite.color = Color::WHITE;
    let mut entity_commands = commands.get_entity(entity).unwrap();
    entity_commands.remove::<SelectedEditorObjectPreview>();
    entity_commands.insert(PlacedObject {});

    commands.trigger(SetSelectedEditorObject {
        key: key.clone(),
        atlas,
    })
}

pub fn editor_load_textures<K: AssetKey + Component>(
    asset_server: Res<AssetServer>,
    sprite_paths: Res<SpritePathMap<K>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut sprite_handle_map: ResMut<SpriteHandleMap<K>>,
    mut sprite_sheet_handle_map: ResMut<SpriteSheetHandleMap<K>>,
) {
    // TODO: Define somewhere which textures are loaded.
    let key: K = unwrap_result!(String::new().try_into());
    let sprite = unwrap_result!(load_sprite(
        key.clone(),
        &sprite_paths,
        &mut sprite_handle_map,
        &asset_server
    ));
    let _ = load_sprite_sheet(
        key,
        &sprite_paths,
        &mut sprite_sheet_handle_map,
        &asset_server,
    );
    egui_user_textures.add_image(sprite);
}

#[derive(Resource)]
pub struct SelectableSprites<K: AssetKey> {
    pub list: Vec<(K, Handle<TextureAtlasLayout>, usize)>,
}

impl<K : AssetKey> Default for SelectableSprites<K> {
    fn default() -> Self {
        Self {
            list : vec![]
        }
    }
}


pub fn editor_add_sprite<K: AssetKey>(
    mut asset_events: EventReader<AssetEvent<SpriteSheet>>,
    sheet_data_assets: Res<Assets<SpriteSheet>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    sprite_sheet_handle_map: Res<SpriteSheetHandleMap<K>>,
    mut selectable_sprites: ResMut<SelectableSprites<K>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                let key = unwrap_option_continue!(sprite_sheet_handle_map.get_key(id)).clone();
                let sheet_data = unwrap_option_continue!(sheet_data_assets.get(*id));
                let mut layout = TextureAtlasLayout::new_empty(sheet_data.size().into());
                for i in 0..sheet_data.len() {
                    let sprite = unwrap_option_continue!(sheet_data.get(i as u32));
                    layout.add_texture(bevy_math::URect::from_corners(
                        sprite.min.into(),
                        sprite.max.into(),
                    ));
                }
                let handle = layouts.add(layout);
                for i in 0..sheet_data.len() {
                    selectable_sprites
                        .list
                        .push((key.clone(), handle.clone(), i));
                }
            }
            _ => {
                continue;
            }
        }
    }
}

#[derive(Event)]
pub struct SaveLevel {}

pub fn save_level<K: AssetKey + Component, S: LevelElement, D: LevelElement>(
    _trigger: Trigger<SaveLevel>,
    query: StaticElementQuery<K>,
) {
    let builder = generate_level_builder::<K, S, D>(&query);
    let r = ron::ser::to_string_pretty(&builder, Default::default()).unwrap();
    std::fs::write("test.ron", r).unwrap();
}

pub type StaticElementQuery<'world, 'state, 'a, K> = Query<
    'world,
    'state,
    (
        &'a Transform,
        &'a K,
        Option<&'a Collider>,
        Option<&'a TextureAtlas>,
    ),
    With<PlacedObject>,
>;

pub fn generate_level_builder<K: AssetKey + Component, S: LevelElement, D: LevelElement>(
    query: &StaticElementQuery<K>,
) -> LevelBuilder<S, D> {
    todo!()
}
