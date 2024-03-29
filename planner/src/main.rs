mod config;
mod display;
mod driver;
mod follower;
mod planner;
mod points;
mod pruner;
mod state;
mod vision;
mod comms;
mod logging;
mod messages {
    pub mod path {
        include!(concat!(env!("OUT_DIR"), "/messages.path.rs"));
    }
    pub mod diagnostic {
        include!(concat!(env!("OUT_DIR"), "/messages.diagnostic.rs"));
    }
    pub mod command {
        include!(concat!(env!("OUT_DIR"), "/messages.commands.rs"));
    }
}

use driver::{IDriver, SerialDriver};
use follower::Follower;
use opencv::{highgui, prelude::*, videoio, Result};
use planner::Planner;
use points::{PointMap, SimplePointMap};
use vision::Vision;

use crate::{comms::NetworkComms, logging::{AggregateLogger, FileLogger, Logger}, messages::diagnostic::Diagnostic, points::Pos, state::DriveState};

const SHOULD_DISPLAY_VIDEO: bool = true;

fn main() -> Result<()> {
    // https://github.com/EmbarkStudios/puffin/tree/main/puffin
    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    println!("Run this to view profiling data:  puffin_viewer --url {server_addr}");
    puffin::set_scopes_on(true);

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
    let network_comms = NetworkComms::new();
    let file_logger = FileLogger::new();
    let mut logger = AggregateLogger::new(vec![Box::new(network_comms), Box::new(file_logger)]);

    let mut current_state = DriveState::default();
    current_state.angle = -3.141 / 2.;
    current_state.pos = Pos { x: 0.1, y: 0.3 };

    loop {
        puffin::GlobalProfiler::lock().new_frame();
        cap.read(&mut frame)?;
        if frame.size()?.width > 0 && SHOULD_DISPLAY_VIDEO {
            highgui::imshow("window", &frame)?;
        }

        let mut new_points = vision.get_points_from_image(&frame);
        point_map.add_points(&mut new_points);
        let filtered = point_map.filter(pruner::get_should_keep_point_predicate());
        let path = planner.find_path(current_state, &point_map);
        let command = follower.command_to_follow_path(&path);
        driver.drive(command);

        logger.send(&path, &new_points, filtered, &Diagnostic::default());

        if SHOULD_DISPLAY_VIDEO {
            let key = highgui::wait_key(10)?;
            if key > 0 && key != 255 {
                break;
            }
        }
    }
    Ok(())
}
