use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::points::{Point, PointMap, Pos};

pub fn points_predicate() -> impl Fn(&Point) -> bool {
    // capture now so that you don't have to recompute it every call
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    move |pnt| should_keep_point(now, pnt)
}

fn should_keep_point(now: Duration, point: &Point) -> bool {
    // may want to have this be smarter about discarding points that are behind the car sooner
    // and ones that may still be useful for later
    point.expire_at > now.as_secs_f64()
}

fn rescale(x: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let from_range = from_max-from_min;
    let to_range = to_max-to_min;
    let t = (x - from_min) / from_range;
    t * to_range + to_min
}

// Get the value the line finder will set expire_at to for new points
pub fn get_point_expiry(pos: Pos, point_map: &dyn PointMap) -> f64 {
    let count_in_grid = point_map.get_count_in_area(pos) as f32;
    let keep_for = Duration::from_secs_f32(rescale(count_in_grid, 0.0, 100.0, 1.5, 0.2));
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    (now + keep_for).as_secs_f64()
}
