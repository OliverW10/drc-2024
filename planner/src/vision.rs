use std::cmp::min;

use opencv::{
    core::Mat,
    imgproc::{find_contours, ContourApproximationModes, RetrievalModes},
    types::VectorOfVectorOfPoint,
};

use crate::{
    planner::DriveState,
    points::{Point, PointMap},
};

pub enum LineType {
    LEFT,
    RIGHT,
}

// stored between frames to reduce memory allocation
pub struct LineFinder {
    contours: VectorOfVectorOfPoint,
    mask: Option<Mat>,
    colour: LineType,
}

impl LineFinder {
    pub fn new(colour: LineType) -> LineFinder {
        LineFinder {
            contours: VectorOfVectorOfPoint::new(),
            mask: None,
            colour: colour,
        }
    }

    pub fn get_points_for_line(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, ()> {
        let _ = find_contours(
            image,
            &mut self.contours,
            RetrievalModes::RETR_EXTERNAL.into(),
            ContourApproximationModes::CHAIN_APPROX_NONE.into(),
            opencv::core::Point { x: 0, y: 0 },
        );
        Err(())
    }
}
pub struct Vision {
    // points: Box<dyn PointMap>,
    left_finder: LineFinder,
    right_finder: LineFinder,
}

impl Vision {
    pub fn new() -> Vision {
        return Vision {
            left_finder: LineFinder::new(LineType::LEFT),
            right_finder: LineFinder::new(LineType::RIGHT),
        };
    }

    pub fn get_points_from_image(&mut self, image: &opencv::core::Mat, point_map: &mut impl PointMap) -> () {
        let _ = self.left_finder.get_points_for_line(image);
        let _ = self.right_finder.get_points_for_line(image);
        // get blue and yellow masks
        // sample points from contours
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
