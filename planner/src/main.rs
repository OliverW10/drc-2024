mod driver;
mod planner;
mod points;
mod vision;
mod messages {
    pub mod path {
        include!(concat!(env!("OUT_DIR"), "/messages.path.rs"));
    }
}

// https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html

use opencv::{highgui, prelude::*, videoio, Result};
use planner::{DriveState, Planner};
use points::SimplePointMap;
use vision::Vision;

const SHOULD_DISPLAY_VIDEO: bool = true;

fn main() -> Result<()> {
    let window_name = "video capture";
    highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)?;
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    let mut frame = Mat::default();

	let mut point_map = SimplePointMap::new();
	let mut vision = Vision::new();
	let planner = Planner::new();

	let current_state = DriveState::default();

    loop {
        cam.read(&mut frame)?;
        if frame.size()?.width > 0 && SHOULD_DISPLAY_VIDEO {
            highgui::imshow(window_name, &frame)?;
        }

		let points = vision.get_points_from_image(&frame, &mut point_map);
		let path = planner.find_path(current_state, &point_map);
        // get command from path
        // send command to controller

        if SHOULD_DISPLAY_VIDEO {
            let key = highgui::wait_key(10)?;
            if key > 0 && key != 255 {
                break;
            }
        }
    }
    Ok(())
}
