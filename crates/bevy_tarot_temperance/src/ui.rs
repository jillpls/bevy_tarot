use bevy_app::{App, Update};
use bevy_ecs::prelude::{Commands, IntoSystemConfigs, Local, Res, ResMut, Resource};
use crate::{SelectableSprites, SetSelectedEditorObject, unwrap_option_continue};
use bevy_egui::egui::Pos2;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_state::condition::in_state;
use bevy_state::prelude::States;
use bevy_tarot_world::magician::AssetKey;
use bevy_tarot_world::magician::bevy_asset::{Assets};
use bevy_tarot_world::magician::bevy_sprite::{TextureAtlas, TextureAtlasLayout};
use bevy_tarot_world::magician::sprite::SpriteHandleMap;

pub fn plugin<S : States + Copy, K : AssetKey>(app: &mut App, state: S) {
    app.add_plugins(EguiPlugin);
    app.insert_resource::<SelectableSprites::<K>>(SelectableSprites::default());
    app.insert_resource(OccupiedScreenSpace::default());
    app.add_systems(Update, editor_ui_system::<K>.run_if(in_state(state)));
}

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

pub fn editor_ui_system<K : AssetKey>(
    mut selected: Local<usize>,
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    selectable_sprites: Res<SelectableSprites<K>>,
    mut commands: Commands,
    sprite_handle_map: Res<SpriteHandleMap<K>>,
) {
    let mut images = {
        let mut result = vec![];
        for (index, (k, layout_handle, i)) in selectable_sprites.list.iter().enumerate() {
            let sprite = unwrap_option_continue!(sprite_handle_map.get(k));
            let img = unwrap_option_continue!(contexts.image_id(&sprite));

            let layout = texture_atlas_layouts.get(layout_handle).unwrap();
            let atlas = TextureAtlas {
                layout: layout_handle.clone(),
                index: *i,
            };

            let rect = unwrap_option_continue!(layout.textures.get(*i));

            let rect = egui::Rect::from_min_max(
                Pos2::new(rect.min.x as f32, rect.min.y as f32),
                Pos2::new(rect.max.x as f32, rect.max.y as f32),
            );
            let uv = egui::Rect::from_min_max(
                egui::pos2(
                    rect.min.x / layout.size.x as f32,
                    rect.min.y / layout.size.y as f32,
                ),
                egui::pos2(
                    rect.max.x / layout.size.x as f32,
                    rect.max.y / layout.size.y as f32,
                ),
            );
            result.push((img, rect, uv, atlas, k));
        }
        result
    };

    let ctx = contexts.ctx_mut();
    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Left resizeable panel");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, (img, rect, uv, atlas, key)) in images.into_iter().enumerate() {
                    let is_currently_selected = *selected == i;
                    let tint = if is_currently_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::DARK_GRAY
                    };
                    if ui
                        .add(
                            egui::ImageButton::new(
                                egui::Image::new(egui::load::SizedTexture::new(
                                    img,
                                    egui::Vec2::new(rect.size().x, rect.size().y),
                                ))
                                    .uv(uv)
                                    .tint(tint),
                            )
                                .selected(is_currently_selected),
                        )
                        .clicked()
                    {
                        *selected = i;
                        commands.trigger(SetSelectedEditorObject {
                            key: key.clone(),
                            atlas: atlas.clone(),
                        })
                    };
                }
                ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
            })
        })
        .response
        .rect
        .width();
}
