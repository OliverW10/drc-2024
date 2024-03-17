use opencv::{core::{perspective_transform, Mat, Point2f, DECOMP_LU}, imgproc::get_perspective_transform};

use crate::points::Pos;



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

pub fn perspective_correct(
    points_ints_in_vec: &Vec<opencv::core::Point2i>,
) -> Result<Vec<Pos>, opencv::Error> {
    puffin::profile_function!();

    // perspective_transform needs a mat with dimentions [number of points, 1, 2] (which isn't documented anywhere and gives bad error messages)
    // and it can't convert noncontinous representations, like vector of vectors, of that into a mat (also give bad error messages)
    let mut x = Vec::<[opencv::core::Point2f; 1]>::new();
    for point in points_ints_in_vec {
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