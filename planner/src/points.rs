use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Copy, Clone, PartialEq, Default)] // needed for copy on DriveState, TODO: do i need Copy on DriveState
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
    pub confidence: f64,
    pub point_type: PointType,
}

pub trait PointMap {
    fn get_points_in_area(&self, around: Pos, max_dist: f64) -> Vec<&Point>;
    fn get_points_below_confidence(&self, cutoff: f64) -> Vec<&Point>;
    fn get_points_lowest_confidence(&self, number: f64) -> Vec<&Point>;
    fn add_points(&mut self, points: &mut Vec<Point>);
    // TODO: make PointMap impl iterator?
    fn filter(&mut self, predicate: fn(&Point) -> bool);
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
        self.all_points
            .iter()
            .filter(|point| point.pos.dist(around) < max_dist)
            .collect()
    }

    fn add_points(&mut self, points: &mut Vec<Point>) {
        self.all_points.append(points);
    }

    fn get_points_below_confidence(&self, cutoff: f64) -> Vec<&Point> {
        vec![]
    }

    fn get_points_lowest_confidence(&self, number: f64) -> Vec<&Point> {
        vec![]
    }

    fn filter(&mut self, predicate: fn(&Point) -> bool){
        // &&Point was auto suggested, not sure why it is double refrence, but if works so idk
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
