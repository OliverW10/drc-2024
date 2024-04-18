use crate::{
    messages::path::SimpleDrive,
    planner::{Path, PathPoint},
    points::Pos,
};

// Currently this is finding a point x meters ahead on the path and pointing the steering at that,
// an alternate approach would be to just use the curvature of the first planned path segment
// however, I think this will perform better as it gives it a bit of a low pass, and will start turning earlier

// In the future I can also make the speed dependent on the curvature at some, further, distance ahead.

fn get_dist_forward_on_path(path: &Path, dist: f64) -> Option<Pos> {
    if path.points.len() <= 2 {
        return None;
    }

    let mut dist = 0.0;
    // if none of the points are far enough to be past the lookahead dist, use the last and second last points
    let mut target_idx = path.points.len() - 2;
    // TODO: can do like zip(path, path[1:])?
    // loop but to second last point
    for point_idx in 0..(path.points.len() - 1) {
        let prev = &path.points[point_idx];
        let next = &path.points[point_idx + 1];
        let cur_dist = prev.pos.dist(next.pos);
        if dist + cur_dist > dist {
            target_idx = point_idx;
            break;
        }
        dist += cur_dist;
    }
    let before = &path.points[target_idx];
    let after = &path.points[target_idx + 1];
    let needed_dist = dist - dist;
    let target = before.pos.dist_along(after.pos, needed_dist);
    Some(target)
}

fn get_curvature_to_target(current: Pos, maybe_target: Option<Pos>) -> f64 {
    match maybe_target {
        None => 0.,
        Some(target) => {
            let delta = target - current;
            delta.y.atan2(delta.x)
        }
    }
}

const LOOKAHEAD_DIST: f64 = 0.3;

pub struct Follower {}

impl Follower {
    pub fn new() -> Follower {
        Follower {}
    }

    pub fn command_to_follow_path(&self, path: &Path) -> SimpleDrive {
        puffin::profile_function!();

        let target_pos = get_dist_forward_on_path(path, LOOKAHEAD_DIST);
        let current_pos = path
            .points
            .first()
            .unwrap_or(&PathPoint {
                pos: Pos { x: 0., y: 0. },
                angle: 0.,
                curvature: 0.,
            })
            .pos;

        // command_to_follow_path
        let result = SimpleDrive {
            curvature: get_curvature_to_target(current_pos, target_pos) as f32,
            speed: 1.0,
        };
        result
    }
}
