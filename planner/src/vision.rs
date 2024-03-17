use opencv::{
    core::{in_range, perspective_transform, BorderTypes, Mat, MatExprTraitConst, MatTraitConst, Point2f, Rect, Size, ToInputArray, VecN, _InputArrayTraitConst, DECOMP_LU}, highgui, imgproc::{
        circle, cvt_color, find_contours, gaussian_blur, get_perspective_transform, ColorConversionCodes, ContourApproximationModes, RetrievalModes
    }, prelude, types::VectorOfVectorOfPoint
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
        border_points.len() > 150
    }

    fn points_from_contours(&self) -> Vec<opencv::core::Point> {
        puffin::profile_function!();

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

const SAMPLE_EVERY: usize = 20;

impl ObjectFinder for LineFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error> {
        puffin::profile_function!();

        {
            puffin::profile_scope!("thresholding");
            in_range(
                // &self.blurred,
                image,
                &self.colour.low,
                &self.colour.high,
                &mut self.mask,
            )?;
        }
        {
            puffin::profile_scope!("contours");
            find_contours(
                &self.mask,
                &mut self.contours,
                RetrievalModes::RETR_EXTERNAL.into(),
                ContourApproximationModes::CHAIN_APPROX_NONE.into(),
                opencv::core::Point { x: 0, y: 0 },
            )?;
        }

        let image_points = self.points_from_contours();
        draw_points_debug(&self.line_type.to_string(), &self.mask, &image_points)?;
        let points = perspective_correct(&image_points)?;

        let time = 0.0; // TODO: get time

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

fn draw_points_debug(wnd_name: &str, mask: &Mat, points: &Vec<opencv::core::Point>) -> Result<(), opencv::Error>{
    puffin::profile_function!();

    let mut display = Mat::default();
    cvt_color(mask, &mut display, ColorConversionCodes::COLOR_GRAY2BGR.into(), 0)?;
    for pnt in points {
        circle(&mut display, *pnt, 3, VecN::<f64, 4> { 0: [0.0, 0.0, 255.0, 0.0] }, -1, opencv::imgproc::LineTypes::FILLED.into(), 0)?;
    }
    highgui::imshow(wnd_name, &display)?;
    Ok(())
}

fn get_perspective_matrix() -> Mat{
    puffin::profile_function!();

    let perspective_points_image = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f {x: -100.0, y: 480.0},
        Point2f {x: 100.0, y: 0.0},
        Point2f {x: 540.0, y: 0.0},
        Point2f {x: 740.0, y: 480.0}
    ]);
    let perspective_points_ground = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f {x: 0.0, y: 100.0},
        Point2f {x: 0.0, y: 0.0},
        Point2f {x: 100.0, y: 0.0},
        Point2f {x: 100.0, y: 100.0}
    ]);
    get_perspective_transform(&perspective_points_image, &perspective_points_ground, DECOMP_LU).unwrap()
}

fn perspective_correct(
    points_ints_in_vec: &Vec<opencv::core::Point2i>,
) -> Result<Vec<Pos>, opencv::Error> {
    puffin::profile_function!();
    // if points_ints_in_vec.len() == 0{
    //     return Ok(vec![]);
    // }
    // TODO: is this double copying all the points?
    let mut x = Vec::<[opencv::core::Point2f; 1]>::new();
    // let mut points = opencv::types::VectorOfVec2f::new();
    for (idx, point) in points_ints_in_vec.iter().enumerate() {
        x.push([opencv::core::Point2f { x: point.x as f32, y: point.y as f32 }]);
    }
    let points = opencv::core::Mat::from_slice_2d(&x)?;
    
    let mut result = opencv::core::Vector::<opencv::core::Point2f>::new();
    // TODO: don't regenerate every frame
    let transform = get_perspective_matrix();
    perspective_transform(&points, &mut result, &transform)?;

    // TODO: unchecked?
    Ok(result.iter().map(|v| Pos {x: v.x as f64, y: v.y as f64}).collect())
}

pub struct Vision {
    point_finders: Vec<Box<dyn ObjectFinder>>,
    cropped: Mat,
    hsv: Mat,
    blurred: Mat,
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
            cropped: Mat::default(),
            hsv: Mat::default(),
            blurred: Mat::default(),
        };
    }

    pub fn get_points_from_image(&mut self, image: &opencv::core::Mat) -> Vec<Point> {
        puffin::profile_function!();

        {
            puffin::profile_scope!("crop");
            let top_crop = 150;
            let size = image.size().unwrap();
            let roi = Rect {x: 0, y: top_crop, width: size.width, height: size.height - top_crop};
            self.cropped = image.apply_1(roi).unwrap();
        }
        // am .unwrap'ing because don't want opencv errors to leak outside of vision
        // and errors should be loud anyway
        {
            puffin::profile_scope!("blur");
            gaussian_blur(
                &self.cropped,
                &mut self.blurred,
                Size::new(3, 3),
                0.0,
                0.0,
                BorderTypes::BORDER_CONSTANT.into(),
            ).unwrap();
        }

        {
            puffin::profile_scope!("hsv");
            cvt_color(
                &self.blurred,
                &mut self.hsv,
                ColorConversionCodes::COLOR_BGR2HSV.into(),
                0,
            ).unwrap();
        }

        // TODO: thread::spawn for each point finder
        self.point_finders
            .iter_mut()
            .flat_map(|finder| finder.get_points(&self.hsv).expect(""))
            .collect()
    }
}
