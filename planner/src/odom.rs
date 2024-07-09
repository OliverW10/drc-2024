use std::{collections::VecDeque, time::{Duration, Instant}};

use crate::{config::file::{Config, ConfigReader}, messages::path::SimpleDrive, points::Pos, state::CarState};


pub trait RelativeStateProvider {
    fn get_movement(&self) -> CarState;
}

pub struct CommandInTime {
    command: SimpleDrive,
    time: Instant,
}

// Returns the movement that is commanded by with a delay 
pub struct BlindRelativeStateProvider {
    delay: Duration,
    commands_queue: VecDeque<CommandInTime>,
    avg: SimpleDrive,
}

impl RelativeStateProvider for BlindRelativeStateProvider {
    fn get_movement(&self) -> CarState {
        puffin::profile_function!();

        let first = self.commands_queue.get(0);
        let second = self.commands_queue.get(1);
        // The differece between second.time and first.time or 0 if either are None
        let cmd_for = second.and_then(|s| first.map(|f| s.time - f.time)).unwrap_or_default();
        
        CarState {
            pos: Pos { x: 0., y: 0. },
            angle: 0.,
            curvature: self.avg.curvature as f64,
            speed: self.avg.speed as f64,
        }.step_time(cmd_for)
    }
}

impl BlindRelativeStateProvider {
    pub fn new() -> BlindRelativeStateProvider {
        BlindRelativeStateProvider {
            delay: Duration::from_secs_f64(0.2),
            commands_queue: VecDeque::new(),
            avg: SimpleDrive { curvature: 0.0, speed: 0.0 },
        }
    }

    pub fn set_command(&mut self, command: SimpleDrive, config: &mut ConfigReader<Config>) {
        self.commands_queue.push_back(CommandInTime { time: Instant::now(), command });

        loop {
            let front = self.commands_queue.front();
            if front.is_none() || front.is_some_and(|cmd| cmd.time.elapsed() < self.delay) {
                break;
            }
            self.commands_queue.pop_front();
        }

        let front_op = self.commands_queue.front();
        if let Some(front) = front_op {
            let cfg_val = config.get_value();
            let current_result = SimpleDrive {
                curvature: front.command.curvature * cfg_val.drive_cfg.odom_turn_fudge,
                speed: front.command.speed * cfg_val.drive_cfg.odom_speed_fudge,
            };
            let alpha = 0.1;
            self.avg = SimpleDrive {
                curvature: self.avg.curvature * (1.0-alpha) + current_result.curvature * alpha,
                speed: self.avg.speed * (1.0-alpha) + current_result.speed * alpha,
            };
        }
    }
}
