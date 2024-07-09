use crate::{
    camera::Recorder, config::file::{Config, ConfigReader}, points::{Point, PointMap}, state::CarState, vision::ObjectFinder
};
use opencv::core::Mat;

// Finds points along just the bottom edge of something
struct ObstacleFinder {
    // obstacle_type: PointType,
    // colour: ColourRange,
}
impl ObstacleFinder {}

impl ObjectFinder for ObstacleFinder {
    fn get_points(
        &mut self, _image: &Mat, _state: &CarState, _: &mut ConfigReader<Config>, _: &dyn PointMap, recorder: &mut Recorder
    ) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
