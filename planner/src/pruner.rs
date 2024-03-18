use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::points::Point;


pub fn should_keep_point(point: &Point) -> bool{
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    // may want to have this be smarter about discarding points that are behind the car sooner
    // and ones that may still be useful for later
    point.expire_at > now.as_secs_f64()
}

pub fn get_line_exiry() -> f64 {
    let keep_for = Duration::from_secs_f64(0.2);
    // TODO: is monotonic?
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + keep_for;
    time_now.as_secs_f64()
}
