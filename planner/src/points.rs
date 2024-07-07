use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops,
};

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl ops::Add for Pos {
    type Output = Pos;
    fn add(self, rhs: Pos) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub for Pos {
    type Output = Pos;
    fn sub(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
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
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
        }
    }

    pub fn dist_along(&self, other: Pos, dist: f64) -> Pos {
        let t = dist / self.dist(other);
        self.lerp(other, t)
    }

    // angle form origin
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl PointType {
    pub fn _is_arrow(&self) -> bool {
        *self == PointType::ArrowLeft || *self == PointType::ArrowRight
    }

    pub fn is_obstacle(&self) -> bool {
        *self == PointType::LeftLine || *self == PointType::RightLine || *self == PointType::Obstacle
    }
}

type PointID = u32;

#[derive(Clone)]
pub struct Point {
    pub pos: Pos,
    pub expire_at: f64,
    pub point_type: PointType,
    pub id: PointID,
}

pub trait PointMap {
    fn get_nearest_point(&self, around: Pos) -> Option<Point>;
    fn get_count_in_area(&self, around: Pos) -> u32;
    fn get_arrow_points(&self) -> Vec<Point>;
    fn add_points(&mut self, points: &Vec<Point>);
    fn remove(&mut self, predicate: &dyn Fn(&Point) -> bool);
    fn get_last_removed_ids(&mut self) -> Vec<PointID>;
}

const GRID_SIZE: f64 = 0.1;

#[derive(PartialEq, Eq, Hash)]
struct GridIndex {
    x: i16,
    y: i16,
}

impl GridIndex {
    fn from_pos(pos: Pos) -> GridIndex {
        GridIndex {
            x: (pos.x / GRID_SIZE).floor() as i16,
            y: (pos.y / GRID_SIZE).floor() as i16,
        }
    }
}

pub struct GridPointMap {
    grid: HashMap<GridIndex, Vec<Point>>,
    arrow_points: Vec<Point>,
    removed_ids: Vec<PointID>,
}

impl GridPointMap {
    pub fn new() -> GridPointMap {
        GridPointMap {
            grid: HashMap::new(),
            arrow_points: Vec::new(),
            removed_ids: Vec::new(),
        }
    }

    fn get_points_in_area(&self, around: Pos, max_dist: f64) -> Vec<Point> {
        let top_left = GridIndex::from_pos(
            around
                + Pos {
                    x: -max_dist,
                    y: -max_dist,
                },
        );
        let bottom_right = GridIndex::from_pos(
            around
                + Pos {
                    x: max_dist,
                    y: max_dist,
                },
        );
    
        let mut result = Vec::new();
        for x in top_left.x..bottom_right.x + 1 {
            for y in top_left.y..bottom_right.y + 1 {
                if let Some(points) = self.grid.get(&GridIndex { x, y }) {
                    let mut points = points
                        .iter()
                        .filter(|point| point.pos.dist(around) < max_dist)
                        .map(|p| p.clone())
                        .collect();
                    result.append(&mut points);
                }
            }
        }
        result
    }
}

impl PointMap for GridPointMap {
    fn get_nearest_point(&self, around: Pos) -> Option<Point> {
        puffin::profile_function!();
        for grid_squares in 1..5 {
            let dist = grid_squares as f64 * GRID_SIZE;
            let points = self.get_points_in_area(around, dist);
            let nearest = points.iter().reduce(|accum: &Point, new: &Point| {
                if new.point_type.is_obstacle() && new.pos.dist(around) < accum.pos.dist(around) {
                    return new;
                } else {
                    return accum;
                }
            });
            if nearest.is_some() {
                return nearest.cloned();
            } 
        }
        None
    }

    fn get_count_in_area(&self, around: Pos) -> u32 {
        return self
            .grid
            .get(&GridIndex::from_pos(around))
            .map_or(0, |x| x.len() as u32);
    }


    fn get_arrow_points(&self) -> Vec<Point> {
        self.arrow_points.clone()
    }

    fn add_points(&mut self, points: &Vec<Point>) {
        puffin::profile_function!();

        for point in points.clone().into_iter() {
            match point.point_type {
                PointType::ArrowLeft | PointType::ArrowRight => {
                    self.arrow_points.push(point);
                }
                _ => {
                    let key = GridIndex::from_pos(point.pos);
                    self.grid.entry(key).or_default().push(point);
                }
            }
        }
    }

    fn remove(&mut self, predicate: &dyn Fn(&Point) -> bool) {
        puffin::profile_function!();

        filter_with_removed(&mut self.arrow_points, predicate, &mut self.removed_ids);

        for points in self.grid.values_mut() {
            filter_with_removed(points, predicate, &mut self.removed_ids);
        }
    }

    fn get_last_removed_ids(&mut self) -> Vec<u32> {
        self.removed_ids.drain(..).collect()
    }
}

// Filters points and records the ids of the removed ones
fn filter_with_removed(points: &mut Vec<Point>, predicate: &dyn Fn(&Point) -> bool, removed: &mut Vec<PointID>) {
    puffin::profile_function!();

    points.retain(|item| {
        if predicate(item) {
            true
        } else {
            removed.push(item.id);
            false
        }
    });
}
