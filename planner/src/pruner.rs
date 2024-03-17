use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::points::Point;


pub fn judge_point_on_age(point: &Point) -> bool{
    let keep_for = Duration::from_secs_f64(1.);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let cutoff = now - keep_for;
    point.confidence < cutoff.as_secs_f64()
}
