use crate::{config::is_running_on_pi, messages::path::SimpleDrive, odom::{BlindRelativeStateProvider, RelativeStateProvider}, points::Pos, state::CarState};
use rppal::pwm::{Channel, Polarity, Pwm};
use serial2::{self, SerialPort};
use std::{collections::VecDeque, time::{Duration, Instant}};

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

impl Drop for PwmDriver {
    fn drop(&mut self) {
        self.stop();
    }
}

const PWM_PERIOD: Duration = Duration::from_millis(20);
const PWM_CENTER: f32 = 1400.0;
const STEER_PWM_MAX: f32 = PWM_CENTER - 400.0;
const STEER_PWM_MIN: f32 = PWM_CENTER + 400.0;
// 75cm right, 85 left (should recalib with better center later) = 1/0.8
const MAX_CURVATURE: f32 = 1.5;
impl Steerer for PwmDriver {
    fn drive_steer(&mut self, curvature: RadiansPerMeter) {
        let t = 0.5 * curvature / MAX_CURVATURE + 0.5;
        let pulse_width_us = STEER_PWM_MIN + t * (STEER_PWM_MAX - STEER_PWM_MIN);
        self.set(pulse_width_us);
    }
}

const MAX_DRIVE_PWM: f32 = 225.0; // 1275, pwm to achive MAX_SPEED
const MIN_DRIVE_PWM: f32 = 125.0; // 1375, highest pwm at which we are stopped
const STOP_DRIVE_PWM: f32 = 1500.0; // pwm to give when wanting to be stopped
const SPEED_DEADZONE: f32 = 0.05;
// Car speed when given MAX_DRIVE_PWM power, speed is assumed to be linear with power below that
// Did 6 meters in 2 10/30 seconds
const MAX_SPEED: f32 = 2.0; //6.0/2.33;

fn get_drive_pwm(speed: MetersPerSecond) -> f32{
    if speed.abs() < SPEED_DEADZONE {
        STOP_DRIVE_PWM
    } else {
        // assumes speeds are symetric around STOP_DRIVE_PWM
        let pwm_travel = MAX_DRIVE_PWM - MIN_DRIVE_PWM;
        let speed_percent = speed.abs() / MAX_SPEED;
        let direction = if speed > 0.0 { -1.0 } else { 1.0 };
        let pulse_width_us = STOP_DRIVE_PWM + direction * (MIN_DRIVE_PWM + pwm_travel * speed_percent);
        pulse_width_us
    }
}

impl Driver for PwmDriver {
    fn drive_speed(&mut self, speed: MetersPerSecond) {
        let pulse_width_us = get_drive_pwm(speed);
        self.set(pulse_width_us);
        // println!("{pulse_width_us}");
    }
}
