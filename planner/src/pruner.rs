use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::points::Point;

pub fn old_points_predicate() -> impl Fn(&Point) -> bool {
    // capture now so that you don't have to recompute it every call
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    move |pnt| should_keep_point(now, pnt)
}

fn should_keep_point(now: Duration, point: &Point) -> bool {
    // may want to have this be smarter about discarding points that are behind the car sooner
    // and ones that may still be useful for later
    point.expire_at > now.as_secs_f64()
}

pub fn get_line_exiry() -> f64 {
    let keep_for = Duration::from_secs_f64(0.2);
    // TODO: is monotonic?
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + keep_for;
    now.as_secs_f64()
}
