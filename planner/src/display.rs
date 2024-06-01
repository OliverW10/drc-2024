use std::time::{SystemTime, UNIX_EPOCH};

use opencv::{
    core::{Mat, MatExprTraitConst, VecN, CV_8UC3},
    highgui,
    imgproc::{circle, line, rectangle},
};

use crate::{
    config::{
        display::SHOULD_DISPLAY_MAP,
        image::{EXCLUDE_RECT, TOP_CROP},
        plan::PLAN_STEP_SIZE_METERS,
    },
    planner::{get_possible_next_states, Path},
    points::{Point, PointType, Pos},
    state::CarState,
};

fn map_to_img(pos: &Pos) -> opencv::core::Point {
    let center_x = 300.;
    let center_y = 300.;
    let scale = 200.; // pixels per meter
    let x = center_x + scale * pos.x;
    let y = center_y + scale * pos.y;
    opencv::core::Point {
        x: x as i32,
        y: y as i32,
    }
}

pub fn draw_map_debug(point_map: &Vec<Point>, path: &Path) {
    puffin::profile_function!();

    if !SHOULD_DISPLAY_MAP {
        return;
    }

    let mut display = Mat::zeros(600, 600, CV_8UC3).unwrap().to_mat().unwrap();
    for pnt in point_map {
        let col = match pnt.point_type {
            PointType::LeftLine => VecN::<f64, 4> {
                0: [255.0, 0.0, 0.0, 0.0],
            },
            PointType::RightLine => VecN::<f64, 4> {
                0: [0.0, 255.0, 255.0, 0.0],
            },
            PointType::Obstacle => VecN::<f64, 4> {
                0: [255.0, 0.0, 255.0, 0.0],
            },
            PointType::ArrowLeft => VecN::<f64, 4> {
                0: [100.0, 100.0, 100.0, 0.0],
            },
            PointType::ArrowRight => VecN::<f64, 4> {
                0: [100.0, 100.0, 100.0, 100.0],
            },
        };
        dot(&mut display, &pnt.pos, col, 1);
    }

    let mut last_pnt = path.points[0].pos;
    let white = VecN::<f64, 4> {
        0: [255., 255., 255., 0.],
    };
    for path_pnt in path.points[1..].iter() {
        // TODO: draw the actual curve rather than just a straight line
        opencv::imgproc::line(
            &mut display,
            map_to_img(&last_pnt),
            map_to_img(&path_pnt.pos),
            white,
            1,
            opencv::imgproc::LineTypes::LINE_8.into(),
            0,
        )
        .unwrap();
        last_pnt = path_pnt.pos.clone();
    }

    let time = 3. * SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let steps = (time % 6.) as i32;

    draw_options(
        &mut display,
        CarState {
            pos: Pos { x: -1., y: 1. },
            angle: -1.1,
            curvature: 0.,
            speed: 1.,
        },
        steps,
    );
    highgui::imshow("map", &display).unwrap();
}

const DISPLAY_ALL_OPTIONS: bool = false;

fn draw_options(img: &mut Mat, start: CarState, depth: i32) {
    if !DISPLAY_ALL_OPTIONS {
        return;
    }

    draw_state_line(img, start);

    for next in get_possible_next_states(start.step_distance(PLAN_STEP_SIZE_METERS)) {
        if depth > 0 {
            draw_options(img, next, depth - 1);
        }
    }
}

const DRAW_RES: u32 = 10;
fn draw_state_line(img: &mut Mat, start: CarState) {
    for dist in (0..DRAW_RES).map(|i| (i as f64 / DRAW_RES as f64) * PLAN_STEP_SIZE_METERS) {
        let prev = start.step_distance(dist);
        let next = start.step_distance(dist + PLAN_STEP_SIZE_METERS / DRAW_RES as f64);
        let white = VecN::<f64, 4> {
            0: [255., 255., 255., 0.],
        };
        opencv::imgproc::line(
            img,
            map_to_img(&prev.pos),
            map_to_img(&next.pos),
            white,
            1,
            opencv::imgproc::LineTypes::LINE_8.into(),
            0,
        )
        .unwrap();
        // dot(img, &prev.pos, white, (5.*dist/PLAN_STEP_SIZE_METERS) as i32);
    }
}

fn dot(img: &mut Mat, p: &Pos, col: VecN<f64, 4>, n: i32) {
    circle(img, map_to_img(p), n, col, -1, opencv::imgproc::LineTypes::FILLED.into(), 0).unwrap();
}

pub fn annotate_video(img: &mut Mat) {
    let white = VecN::<f64, 4> {
        0: [255., 255., 255., 0.],
    };
    let left_pnt = opencv::core::Point { x: 0, y: TOP_CROP };
    let right_pnt = opencv::core::Point { x: 640, y: TOP_CROP };
    rectangle(img, EXCLUDE_RECT, white, 3, opencv::imgproc::LineTypes::LINE_8.into(), 0).unwrap();
    line(img, left_pnt, right_pnt, white, 3, opencv::imgproc::LineTypes::LINE_8.into(), 0).unwrap();
}
