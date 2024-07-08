use crate::{
    camera::Recorder, config::file::ConfigReader, points::{Point, PointMap}, state::CarState, vision::ObjectFinder
};
use opencv::core::Mat;

use super::perspective::PerspectiveTransformPoints;

// Finds points along just the bottom edge of something
struct ObstacleFinder {
    // obstacle_type: PointType,
    // colour: ColourRange,
}
impl ObstacleFinder {}

impl ObjectFinder for ObstacleFinder {
    fn get_points(
        &mut self, _image: &Mat, _state: &CarState, _: &mut ConfigReader<PerspectiveTransformPoints>, _: &dyn PointMap, recorder: &mut Recorder
    ) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
