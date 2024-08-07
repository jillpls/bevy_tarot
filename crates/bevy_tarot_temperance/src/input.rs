use crate::*;
use bevy_app::{App, FixedUpdate, Update};
use bevy_core_pipeline::prelude::Camera2d;
use bevy_ecs::prelude::*;
use bevy_state::prelude::*;
use bevy_tarot_chariot::prelude::{KeyCode, MouseButton};
use bevy_tarot_chariot::{ButtonInput, ButtonMapping, InputAction, MappedButtons};
use bevy_transform::prelude::*;
use bevy_window::prelude::*;
use serde::Serialize;

pub fn plugin<S: States + Copy>(app: &mut App, state: S) {
    app.insert_resource(EditorAction::default_mapping());
    app.add_systems(Update, handle_input.run_if(in_state(state)));
    app.add_systems(FixedUpdate, editor_camera_control.run_if(in_state(state)));
}

/// TODO: DO SOMETHING
const EDGE_DIST: f32 = 50.;

pub fn editor_camera_control(
    mut camera: Query<(&Camera2d, &mut Transform)>,
    q_windows: Query<(&Window), With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    editor_input_mapping: Res<ButtonMapping<EditorAction>>,
) {
    if camera.get_single().is_err() || q_windows.get_single().is_err() {
        return;
    }
    let ((_camera, mut transform), window) = (camera.single_mut(), q_windows.single());
    if let Some(pos) = window.cursor_position() {
        let (height, width) = (window.resolution.height(), window.resolution.width());
        let max_edge_dist = EDGE_DIST / 1920. * width;
        let mut move_x = get_pan_speed_signed(pos.x, width, max_edge_dist) / 10.;
        let mut move_y = -get_pan_speed_signed(pos.y, height, max_edge_dist) / 10.;

        update_move_axis(
            &mut move_x,
            EditorAction::PanRight,
            EditorAction::PanLeft,
            &editor_input_mapping,
            &keyboard_input,
        );
        update_move_axis(
            &mut move_y,
            EditorAction::PanUp,
            EditorAction::PanDown,
            &editor_input_mapping,
            &keyboard_input,
        );

        transform.translation.x += move_x;
        transform.translation.y += move_y;
    }
}

fn handle_input(
    mut commands: Commands,
    kb: Res<ButtonInput<KeyCode>>,
    ms: Res<ButtonInput<MouseButton>>,
    mapping: Res<ButtonMapping<EditorAction>>,
) {
    if kb.pressed(KeyCode::ControlLeft) && kb.just_pressed(KeyCode::KeyS) {
        commands.trigger(SaveLevel {});
    }

    if mapping.just_pressed(&EditorAction::Deselect, Some(&kb), Some(&ms), None) {
        commands.trigger(EditorDeselect {});
    }

    if mapping.just_pressed(&EditorAction::Place, Some(&kb), Some(&ms), None) {
        commands.trigger(EditorPlace {});
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize, Event)]
pub enum EditorAction {
    PanUp,
    PanDown,
    PanLeft,
    PanRight,
    Deselect,
    Place,
}

impl InputAction for EditorAction {
    fn default_mapping() -> ButtonMapping<Self> {
        use EditorAction::*;
        use KeyCode::*;
        let mut mapping = ButtonMapping::<Self>::default();
        mapping.insert_mapping(MappedButtons::new_single(PanUp, KeyW.into()));
        mapping.insert_mapping(MappedButtons::new_single(PanDown, KeyS.into()));
        mapping.insert_mapping(MappedButtons::new_single(PanLeft, KeyA.into()));
        mapping.insert_mapping(MappedButtons::new_single(PanRight, KeyD.into()));
        mapping.insert_mapping(MappedButtons::new_single(Deselect, Escape.into()));
        mapping.insert_mapping(MappedButtons::new_single(Place, MouseButton::Left.into()));
        mapping
    }
}
