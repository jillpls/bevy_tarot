use serde::{Deserialize, Serialize};

/// TODO
#[derive(Serialize, Deserialize)]
pub struct Animation {
    /// TODO
    key_frames: Vec<usize>,
    /// TODO
    behaviour: AnimationBehaviour,
}

/// TODO
#[derive(Default, Serialize, Deserialize)]
pub enum AnimationBehaviour {
    /// TODO
    #[default]
    RunOnce,
    /// TODO
    Loop,
    /// TODO
    Reverse,
}
