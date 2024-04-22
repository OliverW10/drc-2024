use crate::{points::Point, state::CarState, vision::ObjectFinder};

pub struct ArrowFinder {}

impl ArrowFinder {
    pub fn new() -> ArrowFinder {
        ArrowFinder {}
    }
}
impl ObjectFinder for ArrowFinder {
    fn get_points(&mut self, _image: &opencv::core::Mat, state: &CarState) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
