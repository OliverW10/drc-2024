use crate::{messages::path::SimpleDrive, points::Pos, state::CarState};
use rppal::gpio::{Gpio, OutputPin};
use serial2::{self, SerialPort};
use std::time::{Duration, Instant};

pub struct CarCommander {
    driver: Box<dyn Driver>,
    steerer: Box<dyn Steerer>,
    state_provider: BlindRelativeStateProvider,
}

impl CarCommander {
    pub fn new(driver: Box<dyn Driver>, steerer: Box<dyn Steerer>) -> CarCommander {
        CarCommander {
            driver: driver,
            steerer: steerer,
            state_provider: BlindRelativeStateProvider::new(),
        }
    }

    pub fn drive(&mut self, command: SimpleDrive) {
        self.driver.drive_speed(command.speed);
        self.steerer.drive_steer(command.curvature);
        self.state_provider.set_command(command);
    }

    pub fn get_state_provider(&self) -> &impl RelativeStateProvider {
        &self.state_provider
    }
}

type MetersPerSecond = f32;
type RadiansPerMeter = f32;

pub trait Driver {
    fn drive_speed(&mut self, speed: MetersPerSecond);
}

pub trait Steerer {
    fn drive_steer(&mut self, curvature: RadiansPerMeter);
}

pub struct PwmDriver {
    pin: Option<OutputPin>,
}

const ALLOW_NO_GPIO: bool = true;
impl PwmDriver {
    pub fn new() -> PwmDriver {
        // according to rppal docs, software pwm can have jitter of as low as 2us, which is good enough for this
        // the hardware pwm stuff looks annoying, so I'm going with software for now
        // https://docs.rs/rppal/latest/rppal/gpio/index.html
        let pin = Gpio::new().map(|g| g.get(17).unwrap().into_output());
        PwmDriver {
            pin: if ALLOW_NO_GPIO {
                pin.ok()
            } else {
                Some(pin.unwrap())
            },
        }
    }
}

const PWM_PERIOD: Duration = Duration::from_millis(20);
const PWM_MAX: f32 = 2000.0;
const PWM_MIN: f32 = 1000.0;

const MAX_CURVATURE: f32 = 4.0;
impl Steerer for PwmDriver {
    fn drive_steer(&mut self, curvature: RadiansPerMeter) {
        let t = 0.5 * curvature / MAX_CURVATURE + 0.5;
        let pulse_width_us = PWM_MIN + t * (PWM_MAX - PWM_MIN);
        if let Some(pin) = &mut self.pin {
            pin.set_pwm(PWM_PERIOD, Duration::from_micros(pulse_width_us as u64))
                .unwrap();
        }
    }
}

pub struct SerialDriver {
    port_file: Option<SerialPort>,
}

impl SerialDriver {
    pub fn new(name: &str) -> SerialDriver {
        SerialDriver {
            port_file: match SerialPort::open(name, 115200) {
                Ok(file) => {
                    println!("Successfully opened {}", name);
                    Some(file)
                }
                Err(_) => {
                    println!("Could not open {}", name);
                    None
                }
            },
        }
    }
}

const METERS_PER_ROTATION: f32 = 0.1 * 3.141;

impl Driver for SerialDriver {
    fn drive_speed(&mut self, speed: MetersPerSecond) {
        let rps = speed / METERS_PER_ROTATION;
        if let Some(serial) = &self.port_file {
            serial
                .write(format!("v 0 {rps}\nv 1 {rps}\n").as_bytes())
                .unwrap();
        }
    }
}

pub trait RelativeStateProvider {
    fn get_movement(&self) -> CarState;
}

struct BlindRelativeStateProvider {
    last_command: SimpleDrive,
    last_command_at: Instant,
    previous_command_for: Duration,
}

impl RelativeStateProvider for BlindRelativeStateProvider {
    fn get_movement(&self) -> CarState {
        let result = CarState {
            pos: Pos { x: 0., y: 0. },
            angle: 0.,
            curvature: self.last_command.curvature as f64,
            speed: self.last_command.speed as f64,
        }
        .step_time(self.previous_command_for);

        result
    }
}

impl BlindRelativeStateProvider {
    fn new() -> BlindRelativeStateProvider {
        BlindRelativeStateProvider {
            last_command: SimpleDrive::default(),
            last_command_at: Instant::now(),
            previous_command_for: Duration::ZERO,
        }
    }

    fn set_command(&mut self, command: SimpleDrive) {
        self.last_command = command;
        self.previous_command_for = self.last_command_at.elapsed();
        self.last_command_at = Instant::now();
    }
}
