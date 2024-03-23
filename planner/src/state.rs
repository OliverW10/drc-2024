use crate::points::Pos;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct DriveState {
    pub pos: Pos,
    pub angle: f64,     // heading of the car
    pub curvature: f64, // turn angle
    pub speed: f64,
}

impl DriveState {
    pub fn step_distance(&self, dist: f64) -> DriveState {
        return DriveState {
            pos: self.pos + get_along_arc(dist, self.curvature).rotate(self.angle),
            angle: self.angle + self.curvature * dist,
            curvature: self.curvature,
            speed: self.speed,
        };
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
