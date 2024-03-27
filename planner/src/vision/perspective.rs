use opencv::{
    core::{
        perspective_transform, Mat, MatExprTraitConst, Point2f,
        CV_32FC2, DECOMP_LU,
    },
    imgproc::get_perspective_transform
};

use crate::points::Pos;

fn get_perspective_matrix() -> Mat {
    puffin::profile_function!();

    let perspective_points_image = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f { x: -50., y: 480. },
        Point2f { x: 50., y: 0. },
        Point2f { x: 590., y: 0. },
        Point2f { x: 690., y: 480. },
    ]);
    let perspective_points_ground = opencv::core::Vector::<Point2f>::from_iter(vec![
        Point2f { x: 0.0, y: 0.5 },
        Point2f { x: 0.0, y: 0.0 },
        Point2f { x: 0.5, y: 0.0 },
        Point2f { x: 0.5, y: 0.5 },
    ]);
    get_perspective_transform(
        &perspective_points_image,
        &perspective_points_ground,
        DECOMP_LU,
    )
    .unwrap()
}

pub fn perspective_correct(points_ints_in_vec: &Vec<opencv::core::Point2i>) -> Vec<Pos> {
    puffin::profile_function!();

    let mut x = Vec::<opencv::core::Point2f>::new();
    for point in points_ints_in_vec {
        x.push(opencv::core::Point2f {
            x: point.x as f32,
            y: point.y as f32,
        });
    }
    let points = Mat::from_slice(&x).unwrap();

    let mut result_mat = opencv::core::Mat::zeros(x.len() as i32, 2, CV_32FC2)
        .unwrap()
        .to_mat()
        .unwrap();
    // TODO: don't regenerate every frame
    let transform = get_perspective_matrix();
    perspective_transform(&points, &mut result_mat, &transform).unwrap();

    let mut result_final = vec![];
    for p in result_mat.iter::<Point2f>().unwrap() {
        result_final.push(Pos {
            x: p.1.x as f64,
            y: p.1.y as f64,
        });
    }
    result_final
}
