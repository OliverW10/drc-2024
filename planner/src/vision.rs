
use std::cmp::min;

use opencv::{highgui, prelude::*, videoio};
use crate::{planner::DriveState, points::{Point, PointMap}};


struct Vision {
    // internal state
}

impl Vision{
    fn get_points_from_image() -> impl PointMap {
        // get blue and yellow contours
        // sample points from contours
        // perspective correct points


        // get purple mask and do bottom edge filter
        // get contours and samples points
        // perspective correct points


        // look for other cars?

        // detect finish line
    }
}


const NUM_POINTS_LINE: usize = 5000;
// maximum number of points to sample: 10% of available pixels in mask or 500, whichever is smaller
const PERCENT_MIN_SAMPLE_OBSTACLE: f64 = 0.1;
const NUM_ALWAYS_RESAMPLE_LINE: usize = 500;

const NUM_POINTS_OBSTACLE: usize = 200;
const PERCENT_SAMPLE_OBSTACLE: f64 = 0.1;
const NUM_MAX_SAMPLE_OBSTACLE: usize = 50;

fn resample_from_mask(image_taken_from: DriveState, image: Mat, initial_points: Vec<Point>) -> Vec<Point>{
    let num_potential_samples = 2000; // number of 1 pixels in mask
    let samples_to_take = min(num_potential_samples * PERCENT_MIN_SAMPLE_OBSTACLE, NUM_ALWAYS_RESAMPLE_LINE);
    // remove {samples_to_take} point's from initial_points with the lowest point_value()
    // sample points from mask contours
    // perspective correct newly samples points and add to output
    initial_points
}

fn point_value(point: Point, current_state: DriveState) -> f64{
    // how far away/behind
    // how long since seen
    0.0
}
