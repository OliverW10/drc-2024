use std::f64::consts::PI;

use opencv::{
    core::{Mat, MatExprTraitConst, VecN, CV_8UC3},
    highgui,
    imgproc::circle,
};

use crate::{
    config::plan::PLAN_STEP_SIZE_METERS,
    planner::{get_possible_next_states, Path},
    points::{Point, PointType, Pos},
    state::DriveState,
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

pub fn draw_map_debug(point_map: &Vec<&Point>, path: &Path) -> Result<(), opencv::Error> {
    puffin::profile_function!();

    let mut display = Mat::zeros(600, 600, CV_8UC3)?.to_mat()?;
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
        circle(
            &mut display,
            map_to_img(&pnt.pos),
            1,
            col,
            -1,
            opencv::imgproc::LineTypes::FILLED.into(),
            0,
        )?;
    }

    let mut last_pnt = path.points[0];
    let white = VecN::<f64, 4> {
        0: [255., 255., 255., 0.],
    };
    // for path_pnt in path.points[1..].iter() {
    //     opencv::imgproc::line(&mut display, map_to_img(&last_pnt), map_to_img(path_pnt), white, 1, opencv::imgproc::LineTypes::LINE_8.into(), 0)?;
    //     last_pnt = path_pnt.clone();
    // }

    draw_options(
        &mut display,
        DriveState {
            pos: Pos { x: 0., y: 1. },
            angle: -PI / 2.,
            curvature: 0.,
            speed: 1.,
        },
        1,
    );
    highgui::imshow("map", &display)?;
    Ok(())
}

fn draw_options(img: &mut Mat, start: DriveState, depth: i32) {
    draw_state(img, start);

    if depth > 0 {
        for next in get_possible_next_states(start) {
            draw_options(img, next, depth - 1)
        }
    }
}

const DRAW_RES: u32 = 10;
fn draw_state(img: &mut Mat, start: DriveState) {
    for dist in (0..DRAW_RES).map(|i| (i as f64 / DRAW_RES as f64) * PLAN_STEP_SIZE_METERS) {
        let prev = start.step_distance(dist);
        // let next = start.step_distance(dist + PLAN_STEP_SIZE_METERS / DRAW_RES as f64);
        let white = VecN::<f64, 4> {
            0: [255., 255., 255., 0.],
        };
        // opencv::imgproc::line(img, map_to_img(&prev.pos), map_to_img(&next.pos), white, 1, opencv::imgproc::LineTypes::LINE_8.into(), 0).unwrap();
        opencv::imgproc::circle(
            img,
            map_to_img(&prev.pos),
            1,
            white,
            1,
            opencv::imgproc::LineTypes::FILLED.into(),
            0,
        )
        .unwrap();
    }
}
