mod arrow;
mod lines;
mod obstacle;
mod perspective;

use opencv::{
    core::{BorderTypes, Mat, MatTraitConst, Rect, Size},
    imgproc::{cvt_color, gaussian_blur, ColorConversionCodes},
};

use crate::{
    config::colours,
    points::{Point, PointType},
    state::CarState,
    vision::perspective::relative_to_global_point,
};

use self::{arrow::ArrowFinder, lines::LineFinder};

pub trait ObjectFinder {
    fn get_points(&mut self, image: &opencv::core::Mat) -> Result<Vec<Point>, opencv::Error>;
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
        point_finders.push(Box::new(ArrowFinder::new()));

        return Vision {
            point_finders: point_finders,
            cropped: Mat::default(),
            hsv: Mat::default(),
            blurred: Mat::default(),
        };
    }

    pub fn get_points_from_image(
        &mut self,
        image: &opencv::core::Mat,
        state: CarState,
    ) -> Vec<Point> {
        puffin::profile_function!();

        {
            puffin::profile_scope!("crop");
            let top_crop = 150;
            let size = image.size().unwrap();
            let roi = Rect {
                x: 0,
                y: top_crop,
                width: size.width,
                height: size.height - top_crop,
            };
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
            )
            .unwrap();
        }

        {
            puffin::profile_scope!("hsv");
            cvt_color(
                &self.blurred,
                &mut self.hsv,
                ColorConversionCodes::COLOR_BGR2HSV.into(),
                0,
            )
            .unwrap();
        }

        // TODO: thread::spawn for each point finder
        let points: Vec<Point> = self
            .point_finders
            .iter_mut()
            .flat_map(|finder| finder.get_points(&self.hsv).unwrap())
            .map(|p| relative_to_global_point(p, state))
            .collect();
        points
    }
}
