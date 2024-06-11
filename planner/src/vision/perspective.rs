use opencv::{
    core::{perspective_transform, Mat, MatExprTraitConst, Point2f, CV_32FC2, DECOMP_LU},
    imgproc::get_perspective_transform,
};

use crate::{
    config::{file::ConfigReader, image::{EXCLUDE_RECT, TOP_CROP}},
    points::{Point, Pos},
    state::CarState,
};

pub type PerspectiveTransformPoints = (opencv::core::Vector::<Point2f>, opencv::core::Vector::<Point2f>);

pub fn get_perspective_points_config(file_contents: &str) -> PerspectiveTransformPoints {
    let lines: Vec<&str> = file_contents.lines().collect();
    let mut perspective_points_image = opencv::core::Vector::new();
    for line in &lines[1..5] {
        perspective_points_image.push(Point2f {
            x: line.split(',').next().unwrap().trim().parse().unwrap(),
            y: line.split(',').last().unwrap().trim().parse::<f32>().unwrap() - TOP_CROP as f32
        });
    }
    let mut perspective_points_ground = opencv::core::Vector::<Point2f>::new();
    for line in &lines[6..10] {
        perspective_points_ground.push(Point2f {
            x: line.split(',').next().unwrap().trim().parse().unwrap(),
            y: line.split(',').last().unwrap().trim().parse().unwrap()
        });
    }
    (perspective_points_image, perspective_points_ground)
}

pub fn perspective_correct(points_ints_in_vec: &Vec<opencv::core::Point2i>, config: &mut ConfigReader<PerspectiveTransformPoints>) -> Vec<Pos> {
    puffin::profile_function!();

    let mut point_floats_in_vec = Vec::<opencv::core::Point2f>::new();
    for point in points_ints_in_vec {
        if !EXCLUDE_RECT.contains(*point) {
            point_floats_in_vec.push(opencv::core::Point2f {
                x: point.x as f32,
                y: point.y as f32,
            });
        }        
    }

    let points_in_mat = Mat::from_slice(&point_floats_in_vec).unwrap();

    let mut result_mat = opencv::core::Mat::zeros(point_floats_in_vec.len() as i32, 2, CV_32FC2)
        .unwrap()
        .to_mat()
        .unwrap();
    // TODO: don't regenerate every frame
    let (perspective_points_image, perspective_points_ground) = config.get_value();
    let transform = get_perspective_transform(&perspective_points_image, &perspective_points_ground, DECOMP_LU).unwrap();
    perspective_transform(&points_in_mat, &mut result_mat, &transform).unwrap();

    let mut result_final = vec![];
    for p in result_mat.iter::<Point2f>().unwrap() {
        result_final.push(Pos {
            x: p.1.y as f64,
            y: p.1.x as f64,
        });
    }
    result_final
}

pub fn convert_point_relative_to_global(point: Point, car: &CarState) -> Point {
    Point {
        pos: point.pos.rotate(car.angle) + car.pos,
        ..point
    }
}
