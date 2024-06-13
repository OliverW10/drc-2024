use crate::{config::is_running_on_pi, messages::path::SimpleDrive, points::Pos, state::CarState};
use rppal::pwm::{Channel, Polarity, Pwm};
use serial2::{self, SerialPort};
use std::time::{Duration, Instant};

// Interfaces with hardware to drive the car
pub struct CarCommander {
    driver: Box<dyn Driver>,
    steerer: Box<dyn Steerer>,
    state_provider: BlindRelativeStateProvider,
}

impl CarCommander {
    pub fn new() -> CarCommander {
        CarCommander::new_from_components(
            Box::new(PwmDriver::new(PwmPinNumber::Pin12)),
            Box::new(PwmDriver::new(PwmPinNumber::Pin35)),
        )
    }

    fn new_from_components(driver: Box<dyn Driver>, steerer: Box<dyn Steerer>) -> CarCommander {
        CarCommander {
            driver: driver,
            steerer: steerer,
            state_provider: BlindRelativeStateProvider::new(),
        }
    }

    pub fn drive(&mut self, command: SimpleDrive) {
        puffin::profile_function!();

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
    pin: Option<Pwm>,
    enabled: bool,
}

pub enum PwmPinNumber {
    Pin12,
    Pin35,
}

impl PwmPinNumber {
    fn channel(&self) -> Channel {
        match self {
            Self::Pin12 => Channel::Pwm0,
            Self::Pin35 => Channel::Pwm1,
        }
    }
}

impl PwmDriver {
    pub fn new(channel: PwmPinNumber) -> PwmDriver {
        // https://docs.rs/rppal/latest/rppal/pwm/index.html
        let pwm = Pwm::with_period(channel.channel(), PWM_PERIOD, Duration::from_micros(1500), Polarity::Normal, true);

        PwmDriver {
            pin: if Self::should_panic_if_no_gpio() {
                Some(pwm.unwrap())
            } else {
                pwm.ok()
            },
            enabled: false,
        }
    }

    #[inline]
    fn should_panic_if_no_gpio() -> bool {
        is_running_on_pi()
    }

    fn set(&mut self, pulse_width_us: f32) {
        if let Some(pin) = &mut self.pin {
            if !self.enabled {
                pin.enable().unwrap();
                self.enabled = true;
            }
            pin.set_pulse_width(Duration::from_micros(pulse_width_us as u64))
                .unwrap();
        }
    }

    fn stop(&mut self) {
        if let Some(pin) = &mut self.pin {
            if self.enabled {
                pin.disable().unwrap();
                self.enabled = false;
            }
        }
    }
}

const PWM_PERIOD: Duration = Duration::from_millis(20);
const PWM_RANGE: f32 = 0.5;
const PWM_CENTER: f32 = 1500.0;
const STEER_PWM_MAX: f32 = PWM_CENTER + 500.0 * PWM_RANGE;
const STEER_PWM_MIN: f32 = PWM_CENTER - 500.0 * PWM_RANGE;

const MAX_CURVATURE: f32 = 3.0;
impl Steerer for PwmDriver {
    fn drive_steer(&mut self, curvature: RadiansPerMeter) {
        let t = 0.5 * curvature / MAX_CURVATURE + 0.5;
        let pulse_width_us = STEER_PWM_MIN + t * (STEER_PWM_MAX - STEER_PWM_MIN);
        self.set(pulse_width_us);
    }
}

// Rough estimate for MAX_SPEED
const METERS_PER_ROTATION: f32 = 0.1 * 3.141;
const GEAR_RATIO: f32 = 1. / 5.;
const K_V: f32 = 790.0 / 60.0; // kv = rps per volt unloaded
const LOSSES: f32 = 0.6;
const V_BUS: f32 = 3.7 * 2.;
const MAX_SPEED_ESTIMATE: f32 = K_V * V_BUS * LOSSES * GEAR_RATIO * METERS_PER_ROTATION;

const MAX_DRIVE_PWM: f32 = 1700.0;
const STOP_DRIVE_PWM: f32 = 1500.0;
// Car speed when given MAX_DRIVE_PWM power, speed is assumed to be linear with power below that
// To find experimentally
const MAX_SPEED: f32 = 0.5;

impl Driver for PwmDriver {
    fn drive_speed(&mut self, speed: MetersPerSecond) {
        if speed.abs() < 0.05 {
            self.stop();
        } else {
            let pulse_width_us = STOP_DRIVE_PWM + (MAX_DRIVE_PWM - STOP_DRIVE_PWM) * speed / MAX_SPEED;
            self.set(pulse_width_us);
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

impl Driver for SerialDriver {
    fn drive_speed(&mut self, speed: MetersPerSecond) {
        let rps = speed / METERS_PER_ROTATION;
        if let Some(serial) = &self.port_file {
            serial.write(format!("v 0 {rps}\nv 1 {rps}\n").as_bytes()).unwrap();
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
        puffin::profile_function!();

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
