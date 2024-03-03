// a* pathfind up to certain distance away
// heuristic is just 1 - eucledian distance away
// should craft distance carefully from
// - stay away from all points (yellow, blue, purple, red?)
// - avoid turning
// - increase angle on blue points, decrease on yellow
//
// should the state space of the search include speed? or just turning and work out speed from that
// - will have to characterize turning at speed and see if the limits are low enough for it to matter
//
// will need fast position lookup on points (depending on number of points, idk)
// - simple grid (micheal has claimed these can be faster than quad-trees with appropriatly chosen grid size and are much simpler)
//
//

// phase space
// [x pos, y pos, angle, turn angle, speed]
// only control 2 of them, both of which can be sampled very sparsely when path planning
// at each explored position only need to check for
// - turn angle, a few options of + or - (5 maybe?)
// - speed, increase decrease or maintain
// https://arxiv.org/pdf/1105.1186.pdf

// ai planning and control: https://project-archive.inf.ed.ac.uk/ug4/20191552/ug4_proj.pdf

// https://en.wikipedia.org/wiki/Motion_planning

use std::cmp::{Ord, Reverse};
use std::rc::Rc;
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{
    messages::path::SimpleDrive,
    points::{Point, PointMap, Pos},
};

#[derive(Copy, Clone, PartialEq, Default)]
pub struct DriveState {
    pos: Pos,
    angle: f64,     // angle of the car
    curvature: f64, // turn angle
    speed: f64,
}

impl DriveState {
    fn step_distance(&self, dist: f64) -> DriveState {
        if self.curvature.abs() < 1e-3 {
            return DriveState {
                pos: Pos {
                    x: self.pos.x + dist * self.angle.cos(), // TODO: make who method single case
                    y: self.pos.y + dist * self.angle.sin(),
                },
                angle: self.angle,
                curvature: self.curvature,
                speed: self.speed,
            };
        }
        let radius = 1.0 / self.curvature;
        return DriveState {
            pos: Pos {
                x: 1.0,
                y: dist.sin() * radius,
            },
            angle: self.angle + self.curvature * dist,
            curvature: self.curvature,
            speed: self.speed,
        };
    }

    fn step(&self, time: f64) -> DriveState {
        let dist = time * self.speed;
        return self.step_distance(dist);
    }
}

mod distance_calculators {
    use crate::points::Point;

    use super::DriveState;

    pub fn calculate_avoid_edge_weight_for_point(state: DriveState, point: &Point) -> f64 {
        // add weight for being close to the point
        let max_weight = 5.0;
        let start_dist = 0.4;
        let edge_dist = state.pos.dist(point.pos);

        // goes from max_weight when at the edge to 0 when at start_dist away from edge
        let weighting = (start_dist - edge_dist) / start_dist * max_weight;
        if weighting >= 0.0 {
            weighting
        } else {
            0.0
        }
    }

    pub fn calculate_travel_direction_weight_for_point(state: DriveState, point: &Point) -> f64 {
        // add weight for travelling the wrong angular direction around points
        // extra for arrow points and none for obstacle points
        0.0
    }

    pub fn calculate_curvature_weight(state: DriveState) -> f64 {
        // add weighting to enourage taking smoother lines
        state.curvature.powf(1.5) * 0.1
    }
}

// calculates the distance/traversability map used for pathfinding
fn distance(state: DriveState, nearby_points: &Vec<&Point>) -> f64 {
    let mut total_weight = -0.1;
    for point in nearby_points {
        total_weight += distance_calculators::calculate_avoid_edge_weight_for_point(state, point);
        total_weight +=
            distance_calculators::calculate_travel_direction_weight_for_point(state, point);
    }
    total_weight += distance_calculators::calculate_curvature_weight(state);
    total_weight
}

const MAX_CURVATURE: f64 = 1.0 / 0.3;

fn get_possible_next_states(state: DriveState) -> Vec<DriveState> {
    let mut output = Vec::new();
    let turn_options = 3; // per side
    for new_turn_index in -turn_options..turn_options + 1 {
        let new_curvature = MAX_CURVATURE * (new_turn_index as f64 / turn_options as f64);
        let new_drive_state = DriveState {
            curvature: new_curvature,
            ..state
        };
        output.push(new_drive_state.step(0.1));
    }
    output
}

pub struct Path {
    pub points: Vec<Pos>,
}

#[derive(Clone)]
struct PathNodeData {
    pub state: DriveState,
    pub distance: f64,
    pub prev: Rc<PathNode>,
    pub steps: u32,
}

enum PathNode {
    Node(PathNodeData),
    End,
}

impl PartialEq for PathNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for PathNodeData {}
impl PartialOrd for PathNodeData {
    fn partial_cmp(&self, other: &PathNodeData) -> Option<Ordering> {
        let regular_ordering = self
            .distance
            .partial_cmp(&other.distance)
            .expect("should not have NaN distances");
        // order is reversed so that std::BinaryHeap, which is usually a max heap, acts as a min heap
        Some(Reverse(regular_ordering).0)
    }
}
impl Ord for PathNodeData {
    fn cmp(&self, other: &Self) -> Ordering {
        // let ret = self.partial_cmp(other);
        // match ret {
        //     Some(i) => i,
        //     None => Ordering::Equal
        // }
        self.partial_cmp(other).unwrap_or_else(|| Ordering::Equal)
    }
}

pub struct Planner {
    // any internal state
}

const PLAN_STEP_SIZE_SECONDS: f64 = 0.1;
const PLAN_LENGTH_SECONDS: f64 = 3.0;
const PLAN_STEPS: u32 = (PLAN_LENGTH_SECONDS / PLAN_STEP_SIZE_SECONDS) as u32;

impl Planner {
    pub fn new() -> Planner {
        Planner {}
    }

    pub fn find_path(&self, start_state: DriveState, points: &impl PointMap) -> Path {
        // https://doc.rust-lang.org/std/collections/binary_heap/index.html
        let mut open_set = BinaryHeap::new();
        open_set.push(PathNodeData {
            state: start_state,
            distance: 0.0,
            prev: Rc::new(PathNode::End),
            steps: 0,
        });
        while let Some(current) = open_set.pop() {
            let current_rc = Rc::new(&current);

            if current.steps > PLAN_STEPS {
                // plan must be 3 seconds
                return reconstruct_path(current);
            }

            let next_drive_states = get_possible_next_states(current.state).into_iter();
            let relevant_points = points.get_points_in_area(current.state.pos, 0.5); // TODO: magic number
            let get_node_from_state_fn = |state| {
                PathNodeData {
                    state: state,
                    distance: current_rc.distance + distance(state, &relevant_points),
                    prev: Rc::new(PathNode::End), //Rc::clone(&current_rc),
                    steps: current.steps + 1,
                }
            };
            let next_nodes = next_drive_states.map(get_node_from_state_fn);
            open_set.extend(next_nodes);
        }
        let path = Path { points: Vec::new() };
        path
    }
}

fn reconstruct_path(final_node: PathNodeData) -> Path {
    let mut path = Vec::new();
    path.push(final_node.state.pos);
    let mut current = final_node.prev;
    loop {
        match current.as_ref() {
            PathNode::End => break,
            PathNode::Node(node_data) => {
                path.push(node_data.state.pos);
                current = node_data.prev.clone();
            }
        }
    }
    path.reverse();
    Path { points: path }
}

const LOOKAHEAD_DIST: f64 = 0.2;

impl Planner {
    pub fn command_from_path(path: Path) -> SimpleDrive {
        // pure prusuit
        let result = SimpleDrive {
            curvature: 0.0,
            speed: 0.0,
        };
        result
    }
}

fn get_target_on_path(path: Path) -> Option<Pos> {
    if path.points.len() <= 2 {
        return None;
    }

    let mut dist = 0.0;
    // if none of the points are far enough to be past the lookahead dist, use the last and second last points
    let mut target_idx = path.points.len() - 2;
    // TODO: can do like zip(path, path[1:])?
    // loop but to second last point
    for point_idx in 0..(path.points.len() - 1) {
        let prev = path.points[point_idx];
        let next = path.points[point_idx + 1];
        let cur_dist = prev.dist(next);
        if dist + cur_dist > LOOKAHEAD_DIST {
            target_idx = point_idx;
            break;
        }
        dist += cur_dist;
    }
    let before = path.points[target_idx];
    let after = path.points[target_idx + 1];
    let needed_dist = LOOKAHEAD_DIST - dist;
    let target = before.dist_along(after, needed_dist);
    Some(target)
}
