use crate::{messages::path::SimpleDrive, planner::Path};

pub struct Follower {}

impl Follower {
    pub fn new() -> Follower {
        Follower {}
    }

    pub fn command_to_follow_path(&self, path: &Path) -> SimpleDrive {
        puffin::profile_function!();

        let lookahead_count = 5; // 4 * 0.2
        let mut curvature = 0.0;
        for point in &path.points[..lookahead_count] {
            curvature += point.curvature;
        }
        curvature /= lookahead_count as f64;

        let result = SimpleDrive {
            curvature: curvature as f32,
            speed: 0.2,
        };
        result
    }
}
