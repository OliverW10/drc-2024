mod driver;
mod planner;
mod points;
mod vision;
mod follower;
mod arrow;
mod obstacle;
mod config;
mod messages {
    pub mod path {
        include!(concat!(env!("OUT_DIR"), "/messages.path.rs"));
    }
}

use driver::{IDriver, SerialDriver};
use follower::Follower;
use opencv::{highgui, prelude::*, videoio, Result};
use planner::{DriveState, Planner};
use points::SimplePointMap;
use vision::Vision;

const SHOULD_DISPLAY_VIDEO: bool = true;

fn main() -> Result<()> {
    // highgui::named_window("window", highgui::WINDOW_AUTOSIZE)?;
    let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    let opened = videoio::VideoCapture::is_opened(&camera)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut frame = Mat::default();

	let mut point_map = SimplePointMap::new();
	let mut vision = Vision::new();
	let planner = Planner::new();
    let follower = Follower::new();
    let driver = SerialDriver::new();

	let current_state = DriveState::default();

    loop {
        camera.read(&mut frame)?;
        if frame.size()?.width > 0 && SHOULD_DISPLAY_VIDEO {
            highgui::imshow("window", &frame)?;
        }

		vision.update_points_from_image(&frame, &mut point_map);
		let path = planner.find_path(current_state, &point_map);
        let command = follower.command_from_path(path);
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
