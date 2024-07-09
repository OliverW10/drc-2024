use crate::{
    camera::Recorder, config::file::{Config, ConfigReader}, points::{Point, PointMap}, state::CarState, vision::ObjectFinder
};

pub struct ArrowFinder {}

impl ArrowFinder {
    pub fn new() -> ArrowFinder {
        ArrowFinder {}
    }
}
impl ObjectFinder for ArrowFinder {
    fn get_points(
        &mut self, _image: &opencv::core::Mat, _state: &CarState, _: &mut ConfigReader<Config>,
        _: &dyn PointMap, _: &mut Recorder
    ) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
