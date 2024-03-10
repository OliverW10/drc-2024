mod arrow;
mod config;
mod driver;
mod follower;
mod obstacle;
mod planner;
mod points;
mod vision;
mod messages {
    pub mod path {
        include!(concat!(env!("OUT_DIR"), "/messages.path.rs"));
    }
}

use driver::{IDriver, SerialDriver};
use follower::Follower;
use opencv::{highgui, prelude::*, videoio, Result};
use planner::{DriveState, Planner};
use points::{PointMap, SimplePointMap};
use vision::Vision;
use chrono::{DateTime, Utc};
use env_logger::Builder;
use std::io::Write;

const SHOULD_DISPLAY_VIDEO: bool = true;

fn main() -> Result<()> {
    configure_logging();
    let mut cap = videoio::VideoCapture::new(0, videoio::CAP_V4L2)?;
    // cap.set(videoio::CAP_PROP_BUFFERSIZE, 1.0);
    
    let opened = videoio::VideoCapture::is_opened(&cap)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 640.0)?;
    cap.set(videoio::CAP_PROP_FRAME_WIDTH, 480.0)?;
    let mut frame = Mat::default();

    let mut point_map = SimplePointMap::new();
    let mut vision = Vision::new();
    let planner = Planner::new();
    let follower = Follower::new();
    let driver = SerialDriver::new();

    let current_state = DriveState::default();

    loop {
        cap.read(&mut frame)?;
        if frame.size()?.width > 0 && SHOULD_DISPLAY_VIDEO {
            highgui::imshow("window", &frame)?;
        }

        let mut new_points = vision.get_points_from_image(&frame);
        point_map.add_points(&mut new_points);
        let path = planner.find_path(current_state, &point_map);
        let command = follower.command_to_follow_path(path);
        driver.drive(command);

        if SHOULD_DISPLAY_VIDEO {
            let key = highgui::wait_key(10)?;
            if key > 0 && key != 255 {
                break;
            }
        }
    }
    Ok(())
}

// https://github.com/PhilipDaniels/logging_timer/blob/master/examples/logging_demo.rs
// TODO: move out of main.rs
// Just configures logging in such a way that we can see everything.
fn configure_logging() {
    let mut builder = Builder::from_default_env();
    builder.format(|buf, record| {
        let utc: DateTime<Utc> = Utc::now();

        write!(
            buf,
            "{:?} {} [{}] ",
            //utc.format("%Y-%m-%dT%H:%M:%S.%fZ"),
            utc, // same, probably faster?
            record.level(),
            record.target()
        )?;

        match (record.file(), record.line()) {
            (Some(file), Some(line)) => write!(buf, "[{}/{}] ", file, line),
            (Some(file), None) => write!(buf, "[{}] ", file),
            (None, Some(_line)) => write!(buf, " "),
            (None, None) => write!(buf, " "),
        }?;

        writeln!(buf, "{}", record.args())
    });

    builder.init();
}