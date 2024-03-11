use crate::{messages::path::SimpleDrive, planner::Path, points::Pos};

fn get_target_on_path(path: Path) -> Option<Pos> {
    if path.points.len() <= 2 {
        return None;
    }

    let mut dist = 0.0;
    // if none of the points are far enough to be past the lookahead dist, use the last and second last points
    let mut target_idx = path.points.len() - 2;
    // TODO: can do like zip(path, path[1:])?
    // loop but to second last point
    for point_idx in 0..(path.points.len() - 1) {
        let prev = path.points[point_idx];
        let next = path.points[point_idx + 1];
        let cur_dist = prev.dist(next);
        if dist + cur_dist > LOOKAHEAD_DIST {
            target_idx = point_idx;
            break;
        }
        dist += cur_dist;
    }
    let before = path.points[target_idx];
    let after = path.points[target_idx + 1];
    let needed_dist = LOOKAHEAD_DIST - dist;
    let target = before.dist_along(after, needed_dist);
    Some(target)
}

const LOOKAHEAD_DIST: f64 = 0.3;

pub struct Follower {}

impl Follower {
    pub fn new() -> Follower {
        Follower {}
    }

    pub fn command_to_follow_path(&self, path: Path) -> SimpleDrive {
        puffin::profile_function!();

        // command_to_follow_path
        let result = SimpleDrive {
            curvature: 0.0,
            speed: 0.0,
        };
        result
    }
}
