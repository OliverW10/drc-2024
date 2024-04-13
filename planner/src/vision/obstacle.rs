use crate::{
    points::Point,
    vision::ObjectFinder,
};
use opencv::core::Mat;

// Finds points along just the bottom edge of something
struct ObstacleFinder {
    // obstacle_type: PointType,
    // colour: ColourRange,
}
impl ObstacleFinder {}

impl ObjectFinder for ObstacleFinder {
    fn get_points(&mut self, _image: &Mat) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}
