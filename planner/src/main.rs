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

use camera::{Camera, ImageProvider};
use comms::{Commander, NetworkComms};
use driver::{CarCommander, PwmDriver, RelativeStateProvider, SerialDriver};
use follower::Follower;
use logging::{AggregateLogger, FileLogger, Logger};
use messages::{command::CommandMode, diagnostic::Diagnostic, path::SimpleDrive};
use opencv::Result;
use planner::Planner;
use points::{GridPointMap, PointMap, Pos};
use state::CarState;
use std::{collections::VecDeque, time::Instant};
use vision::Vision;

fn main() -> Result<()> {
    // Create objects
    let mut camera = Camera::new();
    let point_map = &mut GridPointMap::new() as &mut dyn PointMap;
    let mut vision = Vision::new();
    let planner = Planner::new();
    let follower = Follower::new();
    let mut driver = CarCommander::new(Box::new(SerialDriver::new("/dev/ttyACM0")), Box::new(PwmDriver::new()));
    let mut network_comms = NetworkComms::new();
    let mut file_logger = FileLogger::new();

    // Initialise state
    let mut current_state = CarState::default();
    current_state.angle = -3.141 / 2.;
    current_state.pos = Pos { x: -2.75, y: 2.0 };
    // current_state.pos = Pos { x: 0.1, y: 0.3 };

    let mut last_frame = Instant::now();
    let mut frame_times = VecDeque::new();
    frame_times.push_back(0 as f32);

    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    println!("Run this to view profiling data:  puffin_viewer --url {server_addr}");
    puffin::set_scopes_on(false);

    loop {
        puffin::GlobalProfiler::lock().new_frame();

        let frame = match camera.get_frame() {
            Some(x) => x,
            None => return Ok(()),
        };

        let movement = driver.get_state_provider().get_movement();
        current_state += movement;

        let network_command = network_comms.get_latest_message();

        let new_points = vision.get_points_from_image(&frame, current_state);

        point_map.add_points(&new_points);

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

        AggregateLogger {
            loggers: vec![&mut network_comms, &mut file_logger],
        }
        .send(
            &path,
            &new_points,
            &point_map.get_last_removed_ids(),
            &get_diagnostic(&frame_times, current_state),
        );

        frame_times.push_front(last_frame.elapsed().as_secs_f32());
        if frame_times.len() > 10 {
            frame_times.pop_back();
        }
        last_frame = Instant::now();
    }
}

fn get_diagnostic(frame_times: &VecDeque<f32>, state: CarState) -> Diagnostic {
    let frametime_avg = frame_times.clone().iter().sum::<f32>() / frame_times.len() as f32;
    let frametime_max = frame_times.clone().into_iter().reduce(f32::max).unwrap();
    Diagnostic {
        actual_speed: state.speed as f32,
        actual_turn: state.curvature as f32,
        framerate_avg: if frametime_avg != 0.0 { 1.0 / frametime_avg } else { 0.0 },
        framerate_90: if frametime_max != 0.0 { 1.0 / frametime_max } else { 0.0 },
    }
}
