use std::collections::HashMap;


#[derive(Copy, Clone, PartialEq)] // needed for copy on DriveState, TODO: do i need Copy on DriveState
pub struct Pos {
    pub x: f64,
    pub y: f64
}

impl Pos {
    pub fn dist(&self, other: Pos) -> f64{
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn lerp(&self, other: Pos, t: f64) -> Pos{
        Pos {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn dist_along(&self, other: Pos, dist: f64) -> Pos{
        let t = dist / self.dist(other);
        self.lerp(other, t)
    }
}

enum PointType {
    LeftLine,
    RightLine,
    Obstacle,
    ArrowLeft,
    ArrowRight
}

pub struct Point {
    pub pos: Pos,
    pub confidence: f64,
    pub point_type: PointType
}

pub trait PointMap {
    fn get_points_in_area(&self, around: Pos, max_dist: f64) -> Vec<Point>;
    fn get_points_
}

pub struct SimplePointMap {
    all_points: Vec<Point>,
}

impl PointMap for SimplePointMap {
    fn get_points(&self, around: Pos, max_dist: f64) -> Vec<Point>{
        let output_points = Vec::new();
        for point in self.all_points{
            if point.dist(around) < max_dist {
                output_points.push(point);
            }
        }
        output_points
    }
}


const GRID_SIZE: f64 = 0.2;

struct GridIndex {
    x: i16,
    y: i16,
}
pub struct GridPointMap{
    grid: HashMap<GridIndex, Vec<Point>>,
}

impl PointMap for GridPointMap {
    fn get_points(&self, around: Pos, max_dist: f64) -> Vec<Point>{
        
    }
}