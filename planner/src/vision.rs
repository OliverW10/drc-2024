use opencv::{
    core::{in_range, BorderTypes, Mat, Size, VecN},
    highgui,
    imgproc::{
        cvt_color, find_contours, gaussian_blur, ColorConversionCodes, ContourApproximationModes,
        RetrievalModes,
    },
    types::VectorOfVectorOfPoint,
};
use rand::Rng;

use crate::{
    config::colours,
    points::{Point, PointType, Pos},
};

pub struct ColourRange {
    pub low: VecN<u8, 3>,
    pub high: VecN<u8, 3>,
}

pub trait ObjectFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error>;
}

// Finds points along the edges of something
pub struct LineFinder {
    line_type: PointType,
    colour: ColourRange,
    // stored between frames to reduce memory allocation
    contours: VectorOfVectorOfPoint,
    mask: Mat,
    blurred: Mat,
}

impl LineFinder {
    pub fn new(obstacle_type: PointType, colour: ColourRange) -> LineFinder {
        LineFinder {
            contours: VectorOfVectorOfPoint::new(),
            mask: Mat::default(),
            blurred: Mat::default(),
            line_type: obstacle_type,
            colour: colour,
        }
    }
    fn is_valid_contour(border_points: &opencv::core::Vector<opencv::core::Point>) -> bool {
        true
    }

    fn points_from_contours(&self) -> Vec<opencv::core::Point> {
        self.contours
            .iter()
            .flat_map(|contour| {
                if LineFinder::is_valid_contour(&contour) {
                    // offset by random amount avoid always sampling the same points along the outline
                    let skip = rand::thread_rng().gen_range(0..SAMPLE_EVERY);
                    contour.iter().skip(skip).step_by(SAMPLE_EVERY).collect() // TODO: try and get this to lazy evaluate
                } else {
                    vec![]
                }
            })
            .collect()
    }
}

const SAMPLE_EVERY: usize = 10;

impl ObjectFinder for LineFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error> {
        gaussian_blur(
            image,
            &mut self.blurred,
            Size::new(5, 5),
            0.0,
            0.0,
            BorderTypes::BORDER_CONSTANT.into(),
        )?;
        in_range(
            &self.blurred,
            &self.colour.low,
            &self.colour.high,
            &mut self.mask,
        )?;
        highgui::imshow(&self.line_type.to_string(), &self.mask)?;

        find_contours(
            &self.mask,
            &mut self.contours,
            RetrievalModes::RETR_EXTERNAL.into(),
            ContourApproximationModes::CHAIN_APPROX_NONE.into(),
            opencv::core::Point { x: 0, y: 0 },
        )?;

        let points = perspective_correct(self.points_from_contours())?;

        let time = 0.0;
        Ok(points
            .iter()
            .map(|p| Point {
                pos: Pos {
                    x: p.x as f64,
                    y: p.y as f64,
                },
                confidence: time,
                point_type: self.line_type,
            })
            .collect())
    }
}

// use opencv vector or Vec?
fn perspective_correct(
    cv_points: Vec<opencv::core::Point>,
) -> Result<Vec<opencv::core::Point>, opencv::Error> {
    // should be few enough points that the allocations are not too big
    let result = opencv::core::Vector::<opencv::core::Point>::new();
    // opencv::calib3d::fisheye_undistort_points_def(cv_points, &mut result, k, d);
    // opencv::core::perspective_transform()
    // result.to_vec()
    Ok(vec![])
}

pub struct Vision {
    point_finders: Vec<Box<dyn ObjectFinder>>,
    hsv: Mat,
}

impl Vision {
    pub fn new() -> Vision {
        let mut point_finders: Vec<Box<dyn ObjectFinder>> = Vec::new();
        point_finders.push(Box::new(LineFinder::new(
            PointType::LeftLine,
            colours::BLUE_MASK,
        )));
        point_finders.push(Box::new(LineFinder::new(
            PointType::RightLine,
            colours::YELLOW_MASK,
        )));
        // point_finders.push(Box::new(ObstacleFinder::new(PointType::Obstacle, colours::PURPLE_MASK)));
        // point_finders.push(Box::new(ObstacleFinder::new(PointType::Obstacle, colours::PURPLE_RED)));
        // point_finders.push(Box::new(ArrowFinder::new()));

        return Vision {
            point_finders: point_finders,
            hsv: Mat::default(),
        };
    }

    pub fn get_points_from_image(&mut self, image: &opencv::core::Mat) -> Vec<Point> {
        // am expect'ing because don't want opencv errors to leak outside of vision
        // and errors should be loud anyway
        cvt_color(
            image,
            &mut self.hsv,
            ColorConversionCodes::COLOR_BGR2HSV.into(),
            0,
        )
        .expect("");

        self.point_finders
            .iter_mut()
            .flat_map(|finder| finder.get_points(&self.hsv).expect(""))
            .collect()
    }
}
