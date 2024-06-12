use opencv::{
    core::{Mat, VecN},
    imgproc::{line, rectangle},
};
use crate::config::image::{EXCLUDE_RECT, TOP_CROP};


pub fn annotate_video(img: &mut Mat) {
    let white = VecN::<f64, 4> {
        0: [255., 255., 255., 0.],
    };
    let left_pnt = opencv::core::Point { x: 0, y: TOP_CROP };
    let right_pnt = opencv::core::Point { x: 640, y: TOP_CROP };
    rectangle(img, EXCLUDE_RECT, white, 3, opencv::imgproc::LineTypes::LINE_8.into(), 0).unwrap();
    line(img, left_pnt, right_pnt, white, 3, opencv::imgproc::LineTypes::LINE_8.into(), 0).unwrap();
}
