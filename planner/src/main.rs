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
mod camera;
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
use opencv::Result;
use planner::Planner;
use points::{PointMap, SimplePointMap};
use vision::Vision;

use crate::{camera::{Camera, ImageProvider}, comms::NetworkComms, logging::{AggregateLogger, FileLogger, Logger}, messages::diagnostic::Diagnostic, points::Pos, state::DriveState};


fn main() -> Result<()> {
    let mut camera = Camera::new();
    let mut point_map = SimplePointMap::new();
    let mut vision = Vision::new();
    let planner = Planner::new();
    let follower = Follower::new();
    let driver = SerialDriver::new();
    let network_comms = Box::new(NetworkComms::new());
    let file_logger = FileLogger::new();
    let mut logger = AggregateLogger::new(vec![network_comms, Box::new(file_logger)]);

    let mut current_state = DriveState::default();
    current_state.angle = -3.141 / 2.;
    current_state.pos = Pos { x: 0.1, y: 0.3 };

    let _ = setup_profiler();

    loop {
        puffin::GlobalProfiler::lock().new_frame();
        
        let frame = match camera.get_frame() {
            Some(x) => x,
            None => return Ok(())
        };

        let mut new_points = vision.get_points_from_image(&frame);

        point_map.add_points(&mut new_points);

        point_map.remove(pruner::old_points_predicate());

        let path = planner.find_path(current_state, &point_map);
        
        let command = follower.command_to_follow_path(&path);

        driver.drive(command);

        logger.send(&path, &new_points, point_map.num_removed(), &Diagnostic::default());
    }
}

fn setup_profiler() -> puffin_http::Server{
    // https://github.com/EmbarkStudios/puffin/tree/main/puffin
    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    println!("Run this to view profiling data:  puffin_viewer --url {server_addr}");
    puffin::set_scopes_on(true);
    _puffin_server
}
