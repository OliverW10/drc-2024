mod camera;
mod comms;
mod config;
mod display;
mod driver;
mod follower;
mod logging;
mod planner;
mod points;
mod pruner;
mod state;
mod vision;
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

use comms::Commander;
use driver::{CarCommander, PwmDriver, RelativeStateProvider, SerialDriver};
use follower::Follower;
use messages::{command::CommandMode, path::SimpleDrive};
use opencv::Result;
use planner::Planner;
use points::{PointMap, SimplePointMap};
use vision::Vision;

use crate::{
    camera::{Camera, ImageProvider},
    comms::NetworkComms,
    logging::{FileLogger, Logger},
    messages::diagnostic::Diagnostic,
    points::Pos,
    state::CarState,
};

fn main() -> Result<()> {
    let mut camera = Camera::new();
    let point_map = &mut SimplePointMap::new() as &mut dyn PointMap;
    let mut vision = Vision::new();
    let planner = Planner::new();
    let follower = Follower::new();
    let mut driver = CarCommander::new(
        Box::new(SerialDriver::new("/dev/ttyACM0")),
        Box::new(PwmDriver::new()),
    );
    let mut network_comms = NetworkComms::new();
    let mut file_logger = FileLogger::new();

    let mut current_state = CarState::default();
    current_state.angle = -3.141 / 2.;
    current_state.pos = Pos { x: 0.1, y: 0.3 };

    let _ = setup_profiler();

    loop {
        puffin::GlobalProfiler::lock().new_frame();

        let frame = match camera.get_frame() {
            Some(x) => x,
            None => return Ok(()),
        };

        current_state += driver.get_state_provider().get_movement();
        // TODO: split things that i want to call multiple times into multiple objects?
        let network_command = network_comms.get_latest_message().unwrap_or_default();

        let mut new_points = vision.get_points_from_image(&frame, current_state);

        point_map.add_points(&mut new_points);

        point_map.remove(&pruner::old_points_predicate());

        let path = planner.find_path(current_state, point_map);

        let command = match CommandMode::try_from(network_command.state).unwrap_or_default() {
            CommandMode::StateAuto => follower.command_to_follow_path(&path),
            CommandMode::StateManual => SimpleDrive {
                speed: network_command.throttle,
                curvature: network_command.turn,
            },
            CommandMode::StateOff => SimpleDrive {
                speed: 0.,
                curvature: 0.,
            },
        };

        driver.drive(command);

        // TODO
        file_logger.send(
            &path,
            &new_points,
            &point_map.get_last_removed_ids(),
            &Diagnostic::default(),
        );
        network_comms.send(
            &path,
            &new_points,
            &point_map.get_last_removed_ids(),
            &Diagnostic::default(),
        );
    }
}

fn setup_profiler() -> puffin_http::Server {
    // https://github.com/EmbarkStudios/puffin/tree/main/puffin
    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    println!("Run this to view profiling data:  puffin_viewer --url {server_addr}");
    puffin::set_scopes_on(true);
    _puffin_server
}
