use opencv::{
    core::{in_range, Mat, VecN},
    highgui,
    imgproc::{circle, cvt_color, find_contours, ColorConversionCodes, ContourApproximationModes, RetrievalModes},
    types::VectorOfVectorOfPoint,
};
use rand::Rng;

use crate::{
    config::{colours::ColourRange, file::ConfigReader},
    points::{Point, PointMap, PointType, Pos},
    pruner::get_point_expiry,
    state::CarState,
    vision::perspective::{convert_point_relative_to_global, perspective_correct},
};

use super::{perspective::PerspectiveTransformPoints, ObjectFinder};

// Finds points along the edges of something
pub struct LineFinder {
    line_type: PointType,
    colour: ColourRange,
    // stored between frames to reduce memory allocation
    contours: VectorOfVectorOfPoint,
    mask: Mat,
}

impl LineFinder {
    pub fn new(obstacle_type: PointType, colour: ColourRange) -> LineFinder {
        LineFinder {
            contours: VectorOfVectorOfPoint::new(),
            mask: Mat::default(),
            line_type: obstacle_type,
            colour: colour,
        }
    }
    fn is_valid_contour(border_points: &opencv::core::Vector<opencv::core::Point>) -> bool {
        border_points.len() > 100
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
    fn get_points(
        &mut self, image: &opencv::core::Mat, state: &CarState, config: &mut ConfigReader<PerspectiveTransformPoints>,
        point_map: &dyn PointMap,
    ) -> Result<Vec<Point>, opencv::Error> {
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
        let points = perspective_correct(&image_points, config);
        draw_mask_debug(&self.line_type.to_string(), &self.mask, &image_points)?;

        Ok(points
            .iter()
            .map(|p| {
                let pos = Pos {
                    x: p.x as f64,
                    y: p.y as f64,
                };
                let confidence = get_point_expiry(pos, point_map);
                convert_point_relative_to_global(
                    Point {
                        pos,
                        expire_at: confidence,
                        point_type: self.line_type,
                        id: rand::random(),
                    },
                    state,
                )
            })
            .collect())
    }
}

const DRAW_MASK: bool = false;

fn draw_mask_debug(wnd_name: &str, mask: &Mat, points_before: &Vec<opencv::core::Point>) -> Result<(), opencv::Error> {
    puffin::profile_function!();

    if !DRAW_MASK {
        return Ok(());
    }
    let mut display = Mat::default();
    cvt_color(mask, &mut display, ColorConversionCodes::COLOR_GRAY2BGR.into(), 0)?;
    for pnt in points_before {
        circle(
            &mut display,
            *pnt,
            3,
            VecN::<f64, 4> {
                0: [0.0, 0.0, 255.0, 0.0],
            },
            -1,
            opencv::imgproc::LineTypes::FILLED.into(),
            0,
        )?;
    }
    highgui::imshow(wnd_name, &display)?;
    Ok(())
}
