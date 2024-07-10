use opencv::{
    core::{in_range, Mat, VecN},
    highgui,
    imgproc::{self, circle, cvt_color, find_contours, ColorConversionCodes, ContourApproximationModes, RetrievalModes},
    types::VectorOfVectorOfPoint,
};
use rand::Rng;

use crate::{
    camera::Recorder, config::file::{Config, ConfigReader, LineColour}, points::{Point, PointMap, PointType, Pos}, pruner::Pruner, state::CarState, vision::perspective::{convert_point_relative_to_global, perspective_correct}
};

use super::ObjectFinder;

// Finds points along the edges of something
pub struct LineFinder {
    line_type: PointType,
    colour: LineColour,
    pruner: Pruner,
    // stored between frames to reduce memory allocation
    contours: VectorOfVectorOfPoint,
    mask: Mat,
    name: String,
}


impl LineFinder {
    pub fn new(obstacle_type: PointType, colour: LineColour, name: String) -> LineFinder {
        LineFinder {
            contours: VectorOfVectorOfPoint::new(),
            mask: Mat::default(),
            line_type: obstacle_type,
            colour: colour,
            pruner: Pruner::new(),
            name: name,
        }
    }
    fn is_valid_contour(border_points: &opencv::core::Vector<opencv::core::Point>, min_border: i32, min_area_ratio: f32) -> bool {
        if border_points.len() < min_border as usize {
            return false;
        }

        let area_ratio = imgproc::contour_area_def(border_points).unwrap_or_default() / (border_points.len() as f64);
        if area_ratio < min_area_ratio as f64{
            return false;
        }

        true
    }

    fn points_from_contours(&self, config: &mut ConfigReader<Config>) -> Vec<opencv::core::Point> {
        puffin::profile_function!();

        let config = config.get_value();

        self.contours
            .iter()
            .flat_map(|contour| {
                if LineFinder::is_valid_contour(&contour, config.contour_cfg.min_boundry, config.contour_cfg.min_area_ratio) {
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
        &mut self, image: &opencv::core::Mat, state: &CarState, config: &mut ConfigReader<Config>,
        point_map: &dyn PointMap, recorder: &mut Recorder
    ) -> Result<Vec<Point>, opencv::Error> {
        puffin::profile_function!();

        {
            puffin::profile_scope!("thresholding");
            let config_val = config.get_value();
            in_range(
                image,
                &config_val.colour_for_line(&self.colour).low,
                &config_val.colour_for_line(&self.colour).high,
                &mut self.mask,
            )?;
        }
        {
            puffin::profile_scope!(format!("save image {}", self.name));
            recorder.record_image(&self.mask, &self.name);
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

        let image_points = self.points_from_contours(config);
        let points = perspective_correct(&image_points, config);
        draw_mask_debug(&self.line_type.to_string(), &self.mask, &image_points)?;

        Ok(points
            .iter()
            .map(|p| {
                let pos = Pos {
                    x: p.x as f64,
                    y: p.y as f64,
                };
                let confidence = self.pruner.get_point_expiry(pos, point_map);
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
