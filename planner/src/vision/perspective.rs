use opencv::{
    core::{perspective_transform, Mat, MatExprTraitConst, Point2f, Rect, CV_32FC2, DECOMP_LU},
    imgproc::get_perspective_transform,
};

use crate::{
    points::{Point, Pos},
    state::CarState,
};

fn get_perspective_matrix() -> Mat {
    puffin::profile_function!();

    let perspective_points_image = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f { x: 259., y: 290. }, // Bottom left
        Point2f { x: 389., y: 290. }, // Bottom right
        Point2f { x: 409., y: 175. }, // Top right
        Point2f { x: 187., y: 175. }, // Top left
    ]);
    let perspective_points_ground = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f { x: -0.11, y: 0.56 },
        Point2f { x: 0.11, y: 0.56 },
        Point2f { x: -0.11, y: 0.26 },
        Point2f { x: 0.11, y: 0.26 },
    ]);
    get_perspective_transform(&perspective_points_image, &perspective_points_ground, DECOMP_LU).unwrap()
}

pub fn perspective_correct(points_ints_in_vec: &Vec<opencv::core::Point2i>) -> Vec<Pos> {
    puffin::profile_function!();

    let mut point_floats_in_vec = Vec::<opencv::core::Point2f>::new();
    for point in points_ints_in_vec {
        point_floats_in_vec.push(opencv::core::Point2f {
            x: point.x as f32,
            y: point.y as f32,
        });
    }
    let points_in_mat = Mat::from_slice(&point_floats_in_vec).unwrap();

    let mut result_mat = opencv::core::Mat::zeros(point_floats_in_vec.len() as i32, 2, CV_32FC2)
        .unwrap()
        .to_mat()
        .unwrap();
    // TODO: don't regenerate every frame
    let transform = get_perspective_matrix();
    perspective_transform(&points_in_mat, &mut result_mat, &transform).unwrap();

    let mut result_final = vec![];
    for p in result_mat.iter::<Point2f>().unwrap() {
        if should_have_point(p.1) {
            result_final.push(Pos {
                x: p.1.y as f64,
                y: p.1.x as f64,
            });
        }
    }
    result_final
}

fn should_have_point(p: opencv::core::Point2f) -> bool{
    let exclude_rects = Rect {x: 200, y: 360, width: 250, height: 120};
    !exclude_rects.contains(opencv::core::Point2i {x:p.x as i32, y:p.y as i32})
}

pub fn convert_point_relative_to_global(point: Point, car: &CarState) -> Point {
    Point {
        pos: point.pos.rotate(car.angle) + car.pos,
        ..point
    }
}
