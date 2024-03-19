use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Copy, Clone, PartialEq, Default, Debug)] // needed for copy on DriveState, TODO: do i need Copy on DriveState
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl Pos {
    pub fn dist(&self, other: Pos) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(&self, other: Pos, t: f64) -> Pos {
        Pos {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn rotate(&self, angle: f64) -> Pos {
        let c = angle.cos();
        let s = angle.sin();
        Pos {
            x: self.x * c + self.y * s,
            y: self.x * s + self.y * c
        }
    }

    pub fn dist_along(&self, other: Pos, dist: f64) -> Pos {
        let t = dist / self.dist(other);
        self.lerp(other, t)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PointType {
    LeftLine,
    RightLine,
    Obstacle,
    ArrowLeft,
    ArrowRight,
}

// https://stackoverflow.com/a/32712140
impl Display for PointType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct Point {
    pub pos: Pos,
    pub expire_at: f64,
    pub point_type: PointType,
}

pub trait PointMap {
    fn get_points_in_area(&self, around: Pos, max_dist: f64) -> Vec<&Point>;
    fn add_points(&mut self, points: &mut Vec<Point>);
    // TODO: make PointMap impl iterator?
    fn filter(&mut self, predicate: impl Fn(&Point) -> bool);
}

pub struct SimplePointMap {
    all_points: Vec<Point>,
}

impl SimplePointMap {
    pub fn new() -> SimplePointMap {
        SimplePointMap {
            all_points: Vec::new(),
        }
    }
}

impl PointMap for SimplePointMap {
    fn get_points_in_area(&self, around: Pos, max_dist: f64) -> Vec<&Point> {
        let ret: Vec<&Point> = self.all_points
            .iter()
            .filter(|point| {
                let result = point.pos.dist(around) < max_dist;
                result
            })
            .collect();
        ret
    }

    fn add_points(&mut self, points: &mut Vec<Point>) {
        puffin::profile_function!();

        self.all_points.append(points);
    }

    fn filter(&mut self, predicate: impl Fn(&Point) -> bool){
        puffin::profile_function!();

        self.all_points = self.all_points.drain(..).filter(predicate).collect();
    }
}

const GRID_SIZE: f64 = 0.2;

struct GridIndex {
    x: i16,
    y: i16,
}
pub struct GridPointMap {
    grid: HashMap<GridIndex, Vec<Point>>,
}

// impl PointMap for GridPointMap {
//     fn get_points(&self, around: Pos, max_dist: f64) -> Vec<Point>{

//     }
// }
