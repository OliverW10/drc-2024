use crate::{config::file::ConfigReader, points::Point, state::CarState, vision::ObjectFinder};

use super::perspective::PerspectiveTransformPoints;

pub struct ArrowFinder {}

impl ArrowFinder {
    pub fn new() -> ArrowFinder {
        ArrowFinder {}
    }
}
impl ObjectFinder for ArrowFinder {
    fn get_points(&mut self, _image: &opencv::core::Mat, _state: &CarState, _: &mut ConfigReader<PerspectiveTransformPoints>) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
