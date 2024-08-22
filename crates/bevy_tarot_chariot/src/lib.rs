//! Input handling utilities for bevy applications.
//!
//! # Example usage:
//! ```
//! use serde::Serialize;
//! use bevy_tarot_chariot::{ButtonMapping, InputAction, MappedButtons};
//! use bevy_tarot_chariot::bevy_input::prelude::*;
//! #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, Serialize)]
//! pub enum SimpleInputAction {
//!     WalkLeft,
//!     WalkRight
//! }
//!
//! impl InputAction for SimpleInputAction {
//!     fn default_mapping() -> ButtonMapping<Self> {
//!         let mut button_mapping = ButtonMapping::default();
//!         button_mapping.insert_mapping(MappedButtons::new(SimpleInputAction::WalkLeft, &[KeyCode::KeyA.into()]));
//!         button_mapping.insert_mapping(MappedButtons::new(SimpleInputAction::WalkRight, &[KeyCode::KeyD.into()]));
//!         button_mapping
//!     }
//! }
//!
    //! // In practice this will happen in a system
//! pub fn main() {
//!     let mapping = SimpleInputAction::default_mapping();
//!     let mut input = ButtonInput::default();
//!     assert!(!mapping.just_pressed(&SimpleInputAction::WalkLeft, Some(&input), None, None));
//!     input.press(KeyCode::KeyA);
//!     assert!(mapping.just_pressed(&SimpleInputAction::WalkLeft, Some(&input), None, None));
//! }
//! ```

use bevy_ecs::prelude::*;
use bevy_input::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
pub use bevy_input;

macro_rules! define_button_count {
    ($value:expr) => {
        /// Default stored space in the SmallVec that holds the buttons. Can be set via feature flags.
        pub const BUTTON_COUNT: usize = $value;
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "4button")] {
        define_button_count!(4);
    } else if #[cfg(feature= "2button")] {
        define_button_count!(2);
    } else if #[cfg(feature = "1button")] {
        define_button_count!(1);
    } else {
        define_button_count!(4);
    }
}

/// Generic abstraction over KeyBoard, Mouse and Gamepad Buttons
/// TODO: What about sticks and stuff
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum GenericButton {
    /// Keyboard Input
    KeyBoard(KeyCode),
    /// Mouse button input
    Mouse(MouseButton),
    /// Gamepad button input
    Gamepad(GamepadButton),
}

impl From<KeyCode> for GenericButton {
    fn from(value: KeyCode) -> Self {
        Self::KeyBoard(value)
    }
}

impl From<MouseButton> for GenericButton {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}

impl From<GamepadButton> for GenericButton {
    fn from(value: GamepadButton) -> Self {
        Self::Gamepad(value)
    }
}

impl GenericButton {
    /// Check if the generic button is currently pressed.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn pressed(
        &self,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        match self {
            GenericButton::KeyBoard(k) => key_codes.map(|bi| bi.pressed(*k)).unwrap_or_default(),
            GenericButton::Mouse(m) => mouse_buttons.map(|bi| bi.pressed(*m)).unwrap_or_default(),
            GenericButton::Gamepad(b) => {
                gamepad_buttons.map(|bi| bi.pressed(*b)).unwrap_or_default()
            }
        }
    }

    /// Check if the generic button was pressed in this cycle.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn just_pressed(
        &self,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        match self {
            GenericButton::KeyBoard(k) => {
                key_codes.map(|bi| bi.just_pressed(*k)).unwrap_or_default()
            }
            GenericButton::Mouse(m) => mouse_buttons
                .map(|bi| bi.just_pressed(*m))
                .unwrap_or_default(),
            GenericButton::Gamepad(b) => gamepad_buttons
                .map(|bi| bi.just_pressed(*b))
                .unwrap_or_default(),
        }
    }

    /// Check if the generic button was released in this cycle.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn just_released(
        &self,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        match self {
            GenericButton::KeyBoard(k) => {
                key_codes.map(|bi| bi.just_released(*k)).unwrap_or_default()
            }
            GenericButton::Mouse(m) => mouse_buttons
                .map(|bi| bi.just_released(*m))
                .unwrap_or_default(),
            GenericButton::Gamepad(b) => gamepad_buttons
                .map(|bi| bi.just_released(*b))
                .unwrap_or_default(),
        }
    }
}

/// Actions that respond to input (and are mapped) need to implement this trait.

pub trait InputAction: Copy + Clone + Hash + Debug + Eq + Serialize {
    /// Default mapping of the actions if it is not loaded manually.
    fn default_mapping() -> ButtonMapping<Self>;
}

/// Maps an action to any amount of buttons.
/// This is optimized for up to 2 mappings.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MappedButtons<A>
where
    A: InputAction + 'static,
{
    /// Action of this mapping.
    action: A,
    /// List of Buttons it maps to. Currently optimized for 2 buttons.
    buttons: SmallVec<[GenericButton; BUTTON_COUNT]>,
}

impl<A: InputAction> MappedButtons<A> {
    /// Simpel Constructor
    pub fn new(action: A, buttons: &[GenericButton]) -> Self {
        let buttons = SmallVec::from_slice(buttons);
        Self { action, buttons }
    }

    /// Initializes `MappedButton<A>` with only a single mapping to `button`
    pub fn new_single(action: A, button: GenericButton) -> Self {
        Self {
            action,
            buttons: SmallVec::from_vec(vec![button]),
        }
    }

    /// List of buttons the action is mapped to.
    pub fn get_buttons(&self) -> &[GenericButton] {
        &self.buttons
    }

    /// Mapped action.

    pub fn get_action(&self) -> A {
        self.action
    }
}

/// Stores mappings of actions to buttons (and reverse)
#[derive(Serialize, Deserialize, Resource, Clone, Debug)]
pub struct ButtonMapping<A: InputAction + 'static> {
    /// Store MappedButtons
    mapped_buttons: Vec<MappedButtons<A>>,
    /// Map action to mapped buttons.
    from_action_map: HashMap<A, usize>,
    /// Map button to objects that map it.
    from_button_map: HashMap<GenericButton, usize>,
}

impl<A: InputAction> Default for ButtonMapping<A> {
    fn default() -> Self {
        Self {
            mapped_buttons: vec![],
            from_action_map: HashMap::new(),
            from_button_map: Default::default(),
        }
    }
}

impl<A: InputAction> ButtonMapping<A> {
    /// Check if the `action` is currently pressed.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn pressed(
        &self,
        action: &A,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        self.get_buttons(action)
            .map(|bts| {
                bts.iter()
                    .any(|b| b.pressed(key_codes, mouse_buttons, gamepad_buttons))
            })
            .unwrap_or_default()
    }

    /// Check if the `action` was pressed in this cycle.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn just_pressed(
        &self,
        action: &A,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        self.get_buttons(action)
            .map(|bts| {
                bts.iter()
                    .any(|b| b.just_pressed(key_codes, mouse_buttons, gamepad_buttons))
            })
            .unwrap_or_default()
    }

    /// Check if the `action` was released in this cycle.
    /// If an optional ButtonInput is omitted it returns false for that button.
    pub fn just_released(
        &self,
        action: &A,
        key_codes: Option<&ButtonInput<KeyCode>>,
        mouse_buttons: Option<&ButtonInput<MouseButton>>,
        gamepad_buttons: Option<&ButtonInput<GamepadButton>>,
    ) -> bool {
        self.get_buttons(action)
            .map(|bts| {
                bts.iter()
                    .any(|b| b.just_released(key_codes, mouse_buttons, gamepad_buttons))
            })
            .unwrap_or_default()
    }

    /// Get the `&MappedButtons<A>` entry for `action` if it exists.
    pub fn get_from_action(&self, action: &A) -> Option<&MappedButtons<A>> {
        self.from_action_map
            .get(action)
            .and_then(|i| self.mapped_buttons.get(*i))
    }

    /// Get the `&MappedButtons<A>` entry for `button` if it exists.
    /// TODO: Allow buttons to be mapped to multiple actions.
    pub fn get_from_button(&self, button: &GenericButton) -> Option<&MappedButtons<A>> {
        self.from_button_map
            .get(button)
            .and_then(|i| self.mapped_buttons.get(*i))
    }

    /// Get the `Action` that the `button` is mapped to.
    pub fn get_action(&self, button: &GenericButton) -> Option<A> {
        self.get_from_button(button).map(|m| m.action)
    }

    /// Get the buttons that the `action` is mapped to.
    pub fn get_buttons(&self, action: &A) -> Option<&[GenericButton]> {
        self.get_from_action(action).map(|m| m.buttons.as_slice())
    }

    /// Check if a `button` is mapped to any action.
    pub fn is_mapped(&self, button: &GenericButton) -> bool {
        self.from_button_map.contains_key(button)
    }

    /// Updates the button mappings for `action`. This replaces the current buttons.
    pub fn update_buttons(&mut self, action: A, buttons: SmallVec<[GenericButton; BUTTON_COUNT]>) {
        if let Some(mapping) = self
            .from_action_map
            .get(&action)
            .and_then(|i| self.mapped_buttons.get_mut(*i))
        {
            let i = self.from_action_map.get(&action).unwrap(); // TODO: this is ugly
            mapping.buttons.iter().for_each(|b| {
                let _ = self.from_button_map.remove(b);
            });
            buttons.iter().for_each(|b| {
                let _ = self.from_button_map.insert(*b, *i);
            });
            self.mapped_buttons.get_mut(*i).unwrap().buttons = buttons; // TODO: Also kinda ugly ngl
        }
    }

    /// Inserts a new mapping and adds the action `A` and the Buttons to internal maps.
    pub fn insert_mapping(&mut self, mapping: MappedButtons<A>) -> bool {
        if self.from_action_map.contains_key(&mapping.action)
            || mapping
                .buttons
                .iter()
                .any(|b| self.from_button_map.contains_key(b))
        {
            return false; // TODO: What do if this happens?
        }
        mapping.buttons.iter().for_each(|b| {
            let _ = self.from_button_map.insert(*b, self.mapped_buttons.len());
        });
        self.from_action_map
            .insert(mapping.action, self.mapped_buttons.len());
        self.mapped_buttons.push(mapping);
        true
    }
}
