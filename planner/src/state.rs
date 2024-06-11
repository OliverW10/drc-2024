use std::{ops, time::Duration};

use crate::points::Pos;

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct CarState {
    pub pos: Pos,
    pub angle: f64,     // heading of the car
    pub curvature: f64, // turn angle
    pub speed: f64,
}

impl CarState {
    pub fn step_distance(&self, dist: f64) -> CarState {
        CarState {
            pos: self.pos + get_along_arc(dist, self.curvature).rotate(self.angle),
            angle: self.angle + self.curvature * dist,
            curvature: self.curvature,
            speed: self.speed,
        }
    }

    pub fn step_time(&self, time: Duration) -> CarState {
        self.step_distance(time.as_secs_f64() * self.speed)
    }
}

impl ops::Add for CarState {
    type Output = CarState;
    fn add(self, rhs: CarState) -> Self::Output {
        CarState {
            pos: self.pos + rhs.pos.rotate(self.angle),
            angle: self.angle + rhs.angle,
            curvature: rhs.curvature,
            speed: rhs.speed,
        }
    }
}

impl ops::AddAssign for CarState {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

fn get_along_arc(dist: f64, curvature: f64) -> Pos {
    if curvature.abs() < 1e-3 {
        Pos { x: dist, y: 0. }
    } else {
        let r = 1. / curvature;
        let angle_around = dist * curvature;
        Pos {
            x: angle_around.sin() * r,
            y: (1. - angle_around.cos()) * r,
        }
    }
}
