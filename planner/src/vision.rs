mod arrow;
mod obstacle;
mod perspective;
mod lines;

use opencv::{
    core::{BorderTypes, Mat, MatExprTraitConst, MatTraitConst, Rect, Size, VecN, CV_8UC3}, highgui, imgproc::{
        circle, cvt_color, gaussian_blur, ColorConversionCodes
    }
};

use crate::{config::colours::{self, ColourRange}, points::{Point, PointType}};

use self::lines::LineFinder;

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
        let points = self.point_finders
            .iter_mut()
            .flat_map(|finder| finder.get_points(&self.hsv).expect(""))
            .collect();
        draw_map_debug(&points);
        points
    }
}


pub fn draw_map_debug(points: &Vec<Point>) -> Result<(), opencv::Error>{
    puffin::profile_function!();

    let mut display = Mat::zeros(600, 600, CV_8UC3)?.to_mat()?;
    let center_x = 300.;
    let center_y = 300.;
    let scale = 800.; // pixels per meter
    for pnt in points {
        let x = center_x + scale * pnt.pos.x;
        let y = center_y + scale * pnt.pos.y;
        let col = match pnt.point_type {
            PointType::LeftLine => VecN::<f64, 4> { 0: [255.0, 0.0, 0.0 ,0.0] },
            PointType::RightLine => VecN::<f64, 4> { 0: [0.0, 255.0, 255.0, 0.0] },
            PointType::Obstacle => VecN::<f64, 4> { 0: [255.0, 0.0, 255.0, 0.0] },
            PointType::ArrowLeft => VecN::<f64, 4> { 0: [100.0, 100.0, 100.0, 0.0] },
            PointType::ArrowRight => VecN::<f64, 4> { 0: [100.0, 100.0, 100.0, 100.0] },
        };
        circle(&mut display, opencv::core::Point {x: x as i32, y: y as i32}, 3, col, -1, opencv::imgproc::LineTypes::FILLED.into(), 0)?;
    }
    highgui::imshow("map", &display)?;
    Ok(())
}
