use crate::{points::Point, vision::ObjectFinder};


struct ArrowFinder {

}
impl ArrowFinder {
    pub fn new() -> ArrowFinder {
        ArrowFinder {}
    }
}
impl ObjectFinder for ArrowFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error> { Ok(vec![]) }
}