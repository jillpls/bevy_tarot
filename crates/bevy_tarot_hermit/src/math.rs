use bevy_math::prelude::*;

/// Checks the distance between a rect and a point.
/// Returns 0. if the point is inside the rect.
pub fn dist_to_rect(rect: &Rect, point: &Vec2) -> f32 {
    if rect.contains(*point) {
        return 0.;
    }
    let dx = (rect.min.x - point.x).max(point.x - rect.max.x);
    let dy = (rect.min.y - point.y).max(point.x - rect.max.x);
    (dx * dx + dy * dy).sqrt()
}
