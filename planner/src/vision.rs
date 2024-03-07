use std::cmp::min;

use opencv::{
    core::{in_range, Mat, Scalar, VecN, CV_8UC1},
    imgproc::{find_contours, threshold, ContourApproximationModes, RetrievalModes},
    types::VectorOfVectorOfPoint,
};
use rand::Rng;

use crate::{
    planner::DriveState,
    points::{Point, PointMap},
};

pub struct ColourRange {
    low: VecN<u8, 3>,
    high: VecN<u8, 3>,
}

mod colours {
    use opencv::core::VecN;

    use super::ColourRange;

    // TODO: use config file
    const fn c<T>(a: T, b: T, c: T) -> VecN<T, 3> {
        VecN::<T, 3> { 0: [a, b, c]}
    }
    const fn r(l: VecN<u8, 3>, h: VecN<u8, 3>) -> ColourRange {
        ColourRange { low: l, high: h}
    }
    pub const YELLOW_MASK: ColourRange = r(c(16, 15, 90), c(45, 255, 255));
    pub const BLUE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}

pub enum ObstacleType {
    LEFT,
    RIGHT,
    ARROW,
    BOX,
    CAR,
}

pub trait ObjectFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error>;
}

// Finds points along the edges of something
pub struct LineFinder {
    obstacle_type: ObstacleType,
    colour: ColourRange,
    // stored between frames to reduce memory allocation
    contours: VectorOfVectorOfPoint,
    mask: Mat,
}

impl LineFinder {
    pub fn new(obstacle_type: ObstacleType, colour: ColourRange) -> LineFinder {
        LineFinder {
            contours: VectorOfVectorOfPoint::new(),
            mask: Mat::default(),
            obstacle_type: obstacle_type,
            colour: colour,
        }
    }
    fn is_valid_contour(border_points: Vector<Point>) -> bool {
        true
    }
}

impl ObjectFinder for LineFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error> {
        in_range(image, &self.colour.low, &self.colour.high, &mut self.mask)?;
        let _ = find_contours(
            &self.mask,
            &mut self.contours,
            RetrievalModes::RETR_EXTERNAL.into(),
            ContourApproximationModes::CHAIN_APPROX_NONE.into(),
            opencv::core::Point { x: 0, y: 0 },
        )?;
        let mut result = vec![];
        const TAKE_EVERY: usize = 10;
        let mut rng = rand::thread_rng();
        for contour in self.contours {
            if LineFinder::is_valid_contour(contour) {
                let skip = rng.gen_range(0..TAKE_EVERY);
                let new_points = contour.iter().skip(skip).step_by(TAKE_EVERY);
                result.extend(new_points);
            }
        }
        Ok(result)
    }
}

// Finds points along just the bottom edge of something
struct ObstacleFinder {

}
impl ObstacleFinder {
    pub fn new(obstacle_type: ObstacleType, colour: ColourRange) -> ObstacleFinder {
        ObstacleFinder {}
    }
}
impl ObjectFinder for ObstacleFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error> { Ok(vec![]) }
}

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

pub struct Vision {
    point_finders: Vec<Box<dyn ObjectFinder>>,
}

impl Vision {
    pub fn new() -> Vision {
        let mut point_finders: Vec<Box<dyn ObjectFinder>> = Vec::new();
        point_finders.push(Box::new(LineFinder::new(ObstacleType::LEFT, colours::BLUE_MASK)));
        point_finders.push(Box::new(LineFinder::new(ObstacleType::RIGHT, colours::YELLOW_MASK)));
        point_finders.push(Box::new(ObstacleFinder::new(ObstacleType::BOX, colours::PURPLE_MASK)));
        point_finders.push(Box::new(ArrowFinder::new()));

        return Vision {
            point_finders: point_finders,
        };
    }

    pub fn update_points_from_image(&mut self, image: &opencv::core::Mat, point_map: &mut impl PointMap) -> () {
        // Find all points in image space
        for mut finder in self.point_finders {
            let mut points = finder.get_points(image).unwrap_or(vec![]);
            point_map.add_points(&mut points);
        }

        // perspective correct points

        // get purple mask and do bottom edge filter
        // get contours and samples points
        // perspective correct points

        // look for other cars?

        // detect finish line
        // self.points.as_ref()
    }
}

// const NUM_POINTS_LINE: usize = 5000;
// maximum number of points to sample: 10% of available pixels in mask or 500, whichever is smaller
const PERCENT_RESAMPLE: f64 = 0.1;
const MAX_RESAMPLE_NUM: u32 = 500;

const NUM_POINTS_OBSTACLE: u32 = 200;
const PERCENT_SAMPLE_OBSTACLE: f64 = 0.1;
const NUM_MAX_SAMPLE_OBSTACLE: u32 = 50;

fn resample_from_mask(
    image_taken_from: DriveState,
    image: Mat,
    initial_points: Vec<Point>,
) -> Vec<Point> {
    let num_potential_samples = 2000.0; // number of white pixels in mask
    let samples_to_take = min(
        (num_potential_samples * PERCENT_RESAMPLE) as u32,
        MAX_RESAMPLE_NUM,
    );
    // remove {samples_to_take} point's from initial_points with the lowest point_value()
    // sample points from mask contours
    // perspective correct newly samples points and add to output
    initial_points
}

fn point_value(point: Point, current_state: DriveState) -> f64 {
    // how far away/behind
    // how long since seen
    0.0
}
