mod arrow;
mod lines;
mod mock;
mod obstacle;
pub mod perspective;

use crate::{
    config::{colours, file::ConfigReader, image::TOP_CROP},
    points::{Point, PointMap, PointType},
    state::CarState,
};
use opencv::{
    core::{BorderTypes, Mat, MatTraitConst, Rect, Size},
    imgproc::{cvt_color, gaussian_blur, ColorConversionCodes},
};
use perspective::PerspectiveTransformPoints;

use self::{arrow::ArrowFinder, lines::LineFinder, mock::FakePointProvider};

pub trait ObjectFinder {
    fn get_points(
        &mut self, image: &opencv::core::Mat, state: &CarState, config: &mut ConfigReader<PerspectiveTransformPoints>,
        point_map: &dyn PointMap,
    ) -> Result<Vec<Point>, opencv::Error>;
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
        point_finders.push(Box::new(LineFinder::new(PointType::LeftLine, colours::BLUE_MASK)));
        point_finders.push(Box::new(LineFinder::new(PointType::RightLine, colours::YELLOW_MASK)));
        // point_finders.push(Box::new(ObstacleFinder::new(PointType::Obstacle, colours::PURPLE_MASK)));
        // point_finders.push(Box::new(ObstacleFinder::new(PointType::Obstacle, colours::PURPLE_RED)));
        point_finders.push(Box::new(ArrowFinder::new()));

        // point_finders.push(Box::new(FakePointProvider {}));

        return Vision {
            point_finders: point_finders,
            cropped: Mat::default(),
            hsv: Mat::default(),
            blurred: Mat::default(),
        };
    }

    // Runs all the vision modules that give their output in map points
    pub fn get_points_from_image(
        &mut self, image: &opencv::core::Mat, state: CarState, config: &mut ConfigReader<PerspectiveTransformPoints>,
        point_map: &dyn PointMap,
    ) -> Vec<Point> {
        puffin::profile_function!();

        {
            puffin::profile_scope!("crop");
            let size = image.size().unwrap();
            let roi = Rect {
                x: 0,
                y: TOP_CROP,
                width: size.width,
                height: size.height - TOP_CROP,
            };
            let cropped_result = image.apply_1(roi);
            if let Ok(cropped) = cropped_result {
                self.cropped = cropped;
            } else {
                println!("Could not crop image, likely an empty image");
                return Vec::new();
            }
            // display_image_and_get_key(&self.cropped);
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
            )
            .unwrap();
        }

        {
            puffin::profile_scope!("hsv");
            cvt_color(&self.blurred, &mut self.hsv, ColorConversionCodes::COLOR_BGR2HSV.into(), 0).unwrap();
        }

        self.point_finders
            .iter_mut()
            .flat_map(|finder| finder.get_points(&self.hsv, &state, config, point_map).unwrap())
            .collect()
    }
}
