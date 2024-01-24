mod planner;

use opencv::{highgui, prelude::*, videoio, Result};

// use crate::vision::Pos;
// use crate::vision::Points;

fn main() -> Result<()> {
	// println!("{}", planner::distance(Pos{x:1.0, y:2.0}, 0.2, 0.1, Points{left_line: vec![], right_line: vec![], obstacle: vec![]}));
	let window = "video capture";
	highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
	let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
	let opened = videoio::VideoCapture::is_opened(&cam)?;
	if !opened {
		panic!("Unable to open default camera!");
	}
	loop {
		let mut frame = Mat::default();
		cam.read(&mut frame)?;
		if frame.size()?.width > 0 {
			highgui::imshow(window, &frame)?;
		}

        // send to controller

		let key = highgui::wait_key(10)?;
		if key > 0 && key != 255 {
			break;
		}
	}
	Ok(())

}
