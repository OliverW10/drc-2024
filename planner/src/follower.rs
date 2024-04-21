use crate::{messages::path::SimpleDrive, planner::Path};

// Currently this is finding a point x meters ahead on the path and pointing the steering at that,
// an alternate approach would be to just use the curvature of the first planned path segment
// however, I think this will perform better as it gives it a bit of a low pass, and will start turning earlier

// In the future I can also make the speed dependent on the curvature at some, further, distance ahead.

pub struct Follower {}

impl Follower {
    pub fn new() -> Follower {
        Follower {}
    }

    pub fn command_to_follow_path(&self, path: &Path) -> SimpleDrive {
        puffin::profile_function!();

        let lookahead = 3;
        let mut curvature = 0.0;
        for point in &path.points[..lookahead] {
            curvature += point.curvature;
        }
        curvature /= lookahead as f64;

        let result = SimpleDrive {
            curvature: curvature as f32,
            speed: 1.0,
        };
        result
    }
}
