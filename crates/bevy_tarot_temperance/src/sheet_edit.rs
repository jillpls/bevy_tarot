//! Define SpriteSheets with debug code

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use bevy_gizmos::gizmos::Gizmos;
use bevy_math::{URect, Vec2};
use bevy_state::prelude::{in_state, States};
use bevy_tarot_hermit::{unwrap_option, unwrap_option_continue, unwrap_result};
use bevy_tarot_world::magician::bevy_asset::{AssetEvent, AssetLoader, Assets, AssetServer, Handle};
use bevy_tarot_world::magician::bevy_render::prelude::Image;
use bevy_tarot_world::magician::bevy_render::texture::ImageLoader;
use bevy_tarot_world::magician::bevy_sprite::{Sprite, SpriteBundle};
use bevy_tarot_world::magician::sprite::{SpriteData, SpriteSheet, SpriteSheetGrid, SpriteSheetLayout};
use log::{info, warn};
use crate::state::TemperanceState;

pub fn plugin<S : States>(app: &mut App, state: S) {
    app.insert_resource(EditingSpriteSheet::default());
    app.observe(update_sprite_sheet);
    app.observe(load_sprite);
    app.add_systems(Update, (draw_sprite_sheet, init_loaded_sprite, ui::sheet_edit_ui).run_if(in_state(state)).run_if(in_state(TemperanceState::SpriteSheetEditor)));
}

#[derive(Resource, Default)]
pub struct EditingSpriteSheet {
    image: Option<Handle<Image>>,
    sheet: Option<SpriteSheet>,
    entity: Option<Entity>
}

#[derive(Event)]
pub enum UpdateSpriteSheet {
    ToGrid(u32, u32),
    ToList,
    GridDimensions(u32, u32),
    AddSprite(URect)
}

fn update_sprite_sheet(trigger: Trigger<UpdateSpriteSheet>, mut sheet: ResMut<EditingSpriteSheet>) {
    let layout = &mut unwrap_option!(&mut sheet.sheet).layout;
    match trigger.event() {
        UpdateSpriteSheet::ToGrid(rows, cols) => {
            match layout {
                SpriteSheetLayout::Grid(_) => { return; }
                SpriteSheetLayout::List(_) => { *layout = SpriteSheetLayout::Grid(SpriteSheetGrid { rows: *rows, cols: *cols  })}
            }
        }
        UpdateSpriteSheet::ToList => {
            match layout {
                SpriteSheetLayout::Grid(_) => { *layout = SpriteSheetLayout::List(vec![])}
                SpriteSheetLayout::List(_) => { return; }
            }
        }
        UpdateSpriteSheet::GridDimensions(rows, cols) => {
            match layout {
                SpriteSheetLayout::Grid(grid) => { grid.rows = *rows; grid.cols = *cols; }
                SpriteSheetLayout::List(_) => { warn!("Tried to set grid dimensions for list layout."); return; }
            }
        }
        UpdateSpriteSheet::AddSprite(rect) => {
            match layout {
                SpriteSheetLayout::Grid(_) => { warn!("Tried to add single sprite to grid layout."); return; }
                SpriteSheetLayout::List(l) => { l.push(SpriteData::new(rect.min.into(), rect.max.into()))}
            }
        }
    }
}

fn tuple_u32_to_vec2(tuple: (u32, u32)) -> Vec2 {
    Vec2::new(tuple.0 as f32, tuple.1 as f32)
}

fn draw_sprite_sheet(sprite_sheet: Res<EditingSpriteSheet>, mut gizmos: Gizmos) {
    let sheet = unwrap_option!(sprite_sheet.sheet.as_ref());
    let size = sheet.size;
    let sheet_size = tuple_u32_to_vec2(size);
    match &sheet.layout {
        SpriteSheetLayout::Grid(grid) => {
            let sprite_size = tuple_u32_to_vec2((size.0 / grid.cols, size.1 / grid.rows));
            for i in 0..grid.cols {
                for j in 0..grid.rows {
                    let pos = Vec2::new(i as f32 * sprite_size.x, j as f32 * sprite_size.y) - (sheet_size - sprite_size)/2. ;
                    gizmos.rect_2d(pos, 0., sprite_size, bevy_color::Color::WHITE)
                }
            }
        }
        SpriteSheetLayout::List(l) => {
            for sprite in l {
                let mut min = tuple_u32_to_vec2(sprite.min) - sheet_size / 2.;
                let mut max = tuple_u32_to_vec2(sprite.max) - sheet_size / 2.;
                min.y = -min.y;
                max.y = -max.y;
                let size = max - min;
                gizmos.rect_2d(min + size / 2., 0., size, bevy_color::Color::WHITE);
            }
        }
    }
}

#[derive(Event)]
pub struct LoadSprite {
    pub path: String
}

fn load_sprite(trigger: Trigger<LoadSprite>, asset_server: Res<AssetServer>, mut sprite_sheet : ResMut<EditingSpriteSheet>) {
    info!("Loading sprite {} into sprite editor.",  Path::new(&trigger.event().path).file_name().unwrap_or_default().to_string_lossy().to_string());
    let handle = asset_server.load::<Image>(&trigger.event().path);
    sprite_sheet.image = Some(handle);
    sprite_sheet.sheet = None;
}

fn init_loaded_sprite(mut commands: Commands, mut asset_events: EventReader<AssetEvent<Image>>, image_assets: Res<Assets<Image>>, mut sprite_sheet: ResMut<EditingSpriteSheet>) {
    for ev in asset_events.read() {
        match ev {
            AssetEvent::LoadedWithDependencies { id } => {
                {
                    if *id != unwrap_option_continue!(&sprite_sheet.image).id() { return; }
                }
                let loaded_image = unwrap_option_continue!(image_assets.get(*id));
                let size = loaded_image.size();
                let new_sprite_sheet = SpriteSheet {
                    layout: SpriteSheetLayout::Grid(SpriteSheetGrid { rows: 1, cols: 1 }),
                    size: (size.x, size.y),
                    grid_sprite_size: Default::default(),
                };
                sprite_sheet.sheet = Some(new_sprite_sheet);
                let e = commands.spawn( SpriteBundle {
                    texture: unwrap_option_continue!(sprite_sheet.image.clone()),
                    ..Default::default()
                });
                sprite_sheet.entity = Some(e.id());
            }
            ev => {
                warn!("{:?}", ev);
                }
        }
    }
}

mod ui {
    use bevy_ecs::prelude::Commands;
    use bevy_ecs::prelude::Local;
    use bevy_egui::{egui, EguiContexts};
    use bevy_egui::egui::{ComboBox};
    use bevy_egui::egui::WidgetType::ComboBox;
    use crate::sheet_edit::UpdateSpriteSheet;

    #[derive(Debug, PartialEq, Default, Copy, Clone)]
    pub enum SheetType {
        #[default]
        Grid,
        List
    }

    pub fn sheet_edit_ui(
        mut commands: Commands,
        mut contexts: EguiContexts,
        mut selected: Local<SheetType>
    ) {
        let prev_selected = *selected;
        let selected = &mut *selected;
        let ctx = contexts.ctx_mut();
        let _ = egui::SidePanel::left("left_panel").resizable(false).show(ctx, |ui| {
            ComboBox::from_label("Sheet Type").selected_text(format!("{:?}", selected)).show_ui(ui, |ui| {
                ui.selectable_value(selected, SheetType::Grid, "Grid");
                ui.selectable_value(selected, SheetType::List, "List");
            });

        });
        if prev_selected != *selected {
            let ev = match selected {
                SheetType::Grid => { UpdateSpriteSheet::ToGrid(1, 1)}
                SheetType::List => { UpdateSpriteSheet::ToList }
            };
            commands.trigger(ev);
        }
    }
}

