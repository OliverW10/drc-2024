use crate::{config::file::ConfigReader, points::Point, state::CarState, vision::ObjectFinder};
use opencv::core::Mat;

use super::perspective::PerspectiveTransformPoints;

// Finds points along just the bottom edge of something
struct ObstacleFinder {
    // obstacle_type: PointType,
    // colour: ColourRange,
}
impl ObstacleFinder {}

impl ObjectFinder for ObstacleFinder {
    fn get_points(&mut self, _image: &Mat, _state: &CarState, config: &mut ConfigReader<PerspectiveTransformPoints>) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
