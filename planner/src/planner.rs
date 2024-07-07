use std::cmp::Ord;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::config::is_running_on_pi;
use crate::points::{Point, PointMap, Pos};
use crate::state::CarState;

pub const PLAN_STEP_SIZE_METERS: f64 = 0.2;
pub const PLAN_MAX_LENGTH_METERS: f64 = 2.0;
const PLAN_MAX_STEPS: u32 = (PLAN_MAX_LENGTH_METERS / PLAN_STEP_SIZE_METERS) as u32;

const MAX_CURVATURE: f64 = 1.8;

mod distance_calculators {
    use std::f64::consts::PI;

    use crate::points::{Point, PointType};

    use super::{CarState, PLAN_STEP_SIZE_METERS};

    const EDGE_MAX_WEIGHT: f64 = 3.0;
    pub const EDGE_MAX_DIST: f64 = 0.25;

    // Weight to make it stay away from the lines
    pub fn calculate_avoid_edge_weight_for_point(state: CarState, point: &Point) -> f64 {
        // add weight for being close to the point
        let edge_dist = state.pos.dist(point.pos);

        // goes from max_weight when at the edge to 0 when at EDGE_MAX_DIST away from edge
        let weighting = (EDGE_MAX_DIST - edge_dist) / EDGE_MAX_DIST * EDGE_MAX_WEIGHT;
        if weighting >= 0.0 {
            weighting
        } else {
            0.0
        }
    }

    // Weight to make it go in the correct direction around points
    // e.g. Drive on the correct side of an arrow point or go the correct direction around the track
    pub fn calculate_angle_change_weight_for_point(state: CarState, arrow: &Point) -> f64 {
        let max_dist = 1.;
        if state.pos.dist(arrow.pos) > max_dist {
            return 0.0;
        }

        let angle_before = (state.pos - arrow.pos).angle();
        let angle_after = (state.step_distance(0.1).pos - arrow.pos).angle();
        let mut angle_diff = angle_after - angle_before;
        // fix angle wrapping
        if angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        if angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        let good_direction = if arrow.point_type == PointType::ArrowLeft {
            -1.0
        } else {
            1.0
        };
        let unweighted = (angle_diff.signum() == good_direction) as i32 as f64;

        unweighted * 0.5
    }

    // Weight to make it take paths with smoother/less turning
    pub fn calculate_curvature_weight(state: CarState) -> f64 {
        // squared so it takes a long shallow turn rather than a short sharp one
        state.curvature.abs().powf(2.0) * 0.4 * PLAN_STEP_SIZE_METERS
        // max is 3^2*0.4*0.2 = 0.72
    }

    // TODO: add weight for begin within ideal distance of nearest point
}

// Calculates the distance/traversability weights used a single step when doing the pathfinding
fn distance(state: CarState, closest_point_to_avoid: Option<Point>, arrow_points: &Vec<Point>) -> f64 {
    puffin::profile_function!();
    // add a slight negative weight so the search behaves closer to a* than bfs
    // this does mean it doesn't guarentee the best path but thats ok beacuse it will be much faster
    let mut total_weight = -PLAN_STEP_SIZE_METERS * 1.0;

    if let Some(point) = closest_point_to_avoid {
        total_weight += distance_calculators::calculate_avoid_edge_weight_for_point(state, &point);
    }

    total_weight += arrow_points
        .iter()
        .map(|p| distance_calculators::calculate_angle_change_weight_for_point(state, p))
        .reduce(f64::max)
        .unwrap_or(0.);

    total_weight += distance_calculators::calculate_curvature_weight(state);

    total_weight
}

pub fn get_possible_next_states(state: CarState) -> Vec<CarState> {
    let mut output = Vec::new();
    let turn_options = 3; // per side
    for new_turn_index in -turn_options..turn_options + 1 {
        let new_curvature = MAX_CURVATURE * (new_turn_index as f64 / turn_options as f64);
        let new_drive_state = CarState {
            curvature: new_curvature,
            ..state
        };
        output.push(new_drive_state);
    }
    output
}

pub struct PathPoint {
    pub pos: Pos,
    pub angle: f64,
    pub curvature: f64,
}

pub struct Path {
    pub points: Vec<PathPoint>,
}

#[derive(Clone)]
struct PathNodeData {
    pub state: CarState,
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

    fn ne(&self, other: &Self) -> bool {
        self.distance != other.distance
    }
}

impl Eq for PathNodeData {}

impl Ord for PathNodeData {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathNodeData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Planner {
    // any internal state
}

impl Planner {
    pub fn new() -> Planner {
        Planner {}
    }

    // Runs Dijkstra's for a time budget and return the best path found
    pub fn find_path(&self, start_state: CarState, points: &dyn PointMap) -> Path {
        puffin::profile_function!();

        let time_budget = if is_running_on_pi() {
            Duration::from_millis(30)
        } else {
            Duration::from_millis(5)
        };
        let started = Instant::now();

        let starting_node = PathNodeData {
            state: start_state,
            distance: 0.0,
            prev: Rc::new(PathNode::End),
            steps: 0,
        };

        let mut best_path = starting_node.clone();

        let mut open_set = BinaryHeap::new();
        open_set.push(starting_node);
        let mut total_paths = 0;

        while let Some(current) = open_set.pop() {
            total_paths += 1;
            let current_rc = Rc::new(PathNode::Node(current.clone()));

            let is_longest_path_so_far = current.steps > best_path.steps;
            let is_better_than_longest_path = current.steps == best_path.steps && current.distance < best_path.distance;
            if is_longest_path_so_far || is_better_than_longest_path {
                best_path = current.clone();
            }
            if started.elapsed() > time_budget {
                break;
            }
            if current.steps >= PLAN_MAX_STEPS {
                continue;
            }

            let next_drive_states = get_possible_next_states(current.state);
            let arrow_points = points.get_arrow_points();
            for next_state_before in next_drive_states {
                let relevant_points = points.get_nearest_point(current.state.pos);
                let next_state = next_state_before.step_distance(PLAN_STEP_SIZE_METERS);
                open_set.push(PathNodeData {
                    state: next_state,
                    distance: current.distance + distance(next_state, relevant_points, &arrow_points),
                    prev: current_rc.clone(),
                    steps: current.steps + 1,
                })
            }
        }

        // println!(
        //     "final path cost: {}, evaluated {} paths in {}ms",
        //     best_path.distance,
        //     total_paths,
        //     started.elapsed().as_millis(),
        // );
        let final_path = reconstruct_path(best_path);
        final_path
    }
}

fn reconstruct_path(final_node: PathNodeData) -> Path {
    puffin::profile_function!();

    let mut path = Vec::new();
    path.push(PathPoint {
        pos: final_node.state.pos,
        angle: final_node.state.angle,
        curvature: final_node.state.curvature,
    });
    let mut current = final_node.prev;
    loop {
        match current.as_ref() {
            PathNode::End => break,
            PathNode::Node(node_data) => {
                path.push(PathPoint {
                    pos: node_data.state.pos,
                    angle: node_data.state.angle,
                    curvature: node_data.state.curvature,
                });
                current = node_data.prev.clone();
            }
        }
    }
    path.reverse();
    Path { points: path }
}
