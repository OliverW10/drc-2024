use crate::{points::{Point, PointType}, vision::{ColourRange, ObjectFinder}};
use opencv::core::Mat;


// Finds points along just the bottom edge of something
struct ObstacleFinder {
    obstacle_type: PointType, colour: ColourRange
}
impl ObstacleFinder {
}

impl ObjectFinder for ObstacleFinder {
    fn get_points(&mut self, image: &Mat) -> Result<Vec<Point>, opencv::Error> {
        Ok(vec![])
    }
}