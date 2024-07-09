use std::{collections::VecDeque, time::{Duration, Instant}};

use crate::{messages::path::SimpleDrive, points::Pos, state::CarState};


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
}

const TURN_FUDGE: f64 = 1.0;
const DRIVE_FUDGE: f64 = 1.0;

impl RelativeStateProvider for BlindRelativeStateProvider {
    fn get_movement(&self) -> CarState {
        puffin::profile_function!();

        let first = self.commands_queue.get(0);
        let second = self.commands_queue.get(1);
        let cmd = first.map(|x| x.command.clone()).unwrap_or_default();
        // The differece between second.time and first.time or 0 if either are None
        let cmd_for = second.and_then(|s| first.map(|f| s.time - f.time)).unwrap_or_default();

        CarState {
            pos: Pos { x: 0., y: 0. },
            angle: 0.,
            curvature: cmd.curvature as f64 * TURN_FUDGE,
            speed: cmd.speed as f64 * DRIVE_FUDGE,
        }.step_time(cmd_for)
    }
}

impl BlindRelativeStateProvider {
    pub fn new() -> BlindRelativeStateProvider {
        BlindRelativeStateProvider {
            delay: Duration::from_secs_f64(0.2),
            commands_queue: VecDeque::new(),
        }
    }

    pub fn set_command(&mut self, command: SimpleDrive) {
        self.commands_queue.push_back(CommandInTime { time: Instant::now(), command });

        loop {
            let front = self.commands_queue.front();
            if front.is_none() || front.is_some_and(|cmd| cmd.time.elapsed() < self.delay) {
                break;
            }
            self.commands_queue.pop_front();
        }
    }
}
