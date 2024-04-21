// https://arxiv.org/pdf/1105.1186.pdf

// ai planning and control: https://project-archive.inf.ed.ac.uk/ug4/20191552/ug4_proj.pdf

// https://en.wikipedia.org/wiki/Motion_planning

use std::cmp::Ord;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::config::plan::{MAX_CURVATURE, PLAN_MAX_STEPS, PLAN_STEP_SIZE_METERS};
use crate::display::draw_map_debug;
use crate::planner::distance_calculators::EDGE_MAX_DIST;
use crate::points::{Point, PointMap, Pos};
use crate::state::CarState;

mod distance_calculators {
    use crate::{config::plan::PLAN_STEP_SIZE_METERS, points::Point};

    use super::CarState;

    pub const EDGE_MAX_DIST: f64 = 0.3;
    pub fn calculate_avoid_edge_weight_for_point(state: CarState, point: &Point) -> f64 {
        // add weight for being close to the point
        let max_weight = 3.0;
        let edge_dist = state.pos.dist(point.pos);

        // goes from max_weight when at the edge to 0 when at EDGE_MAX_DIST away from edge
        let weighting = (EDGE_MAX_DIST - edge_dist) / EDGE_MAX_DIST * max_weight;
        if weighting >= 0.0 {
            weighting
        } else {
            0.0
        }
    }

    pub fn calculate_travel_direction_weight_for_point(state: CarState, point: &Point) -> f64 {
        // add weight for travelling the wrong angular direction around points
        // extra for arrow points and none for obstacle points
        0.0
    }

    pub fn calculate_curvature_weight(state: CarState) -> f64 {
        // add weighting to enourage taking smoother lines
        state.curvature.abs().powf(2.0) * 0.4 * PLAN_STEP_SIZE_METERS
    }
}

// Calculates the distance/traversability weights used for pathfinding
fn distance(state: CarState, nearby_points: &Vec<Point>) -> f64 {
    puffin::profile_function!();
    let mut total_weight = -PLAN_STEP_SIZE_METERS * 1.0;

    total_weight += nearby_points
        .iter()
        .map(|p| distance_calculators::calculate_avoid_edge_weight_for_point(state, p))
        .reduce(f64::max)
        .unwrap_or(0.);

    total_weight += nearby_points
        .iter()
        .map(|p| distance_calculators::calculate_travel_direction_weight_for_point(state, p))
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
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
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

    pub fn find_path(&self, start_state: CarState, points: &dyn PointMap) -> Path {
        puffin::profile_function!();

        let time_budget = Duration::from_millis(3); // for laptop
        // let time_budget = Duration::from_millis(100); // for pi
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
            
            if current.steps > best_path.steps || (current.steps == best_path.steps && current.distance < best_path.distance) {
                best_path = current.clone();
            }
            if started.elapsed() > time_budget {
                break;
            }
            if current.steps > PLAN_MAX_STEPS {
                continue
            }

            let next_drive_states = get_possible_next_states(current.state);
            let relevant_points = points.get_points_in_area(current.state.pos, EDGE_MAX_DIST);
            for next_state in next_drive_states {
                open_set.push(PathNodeData {
                    state: next_state.step_distance(PLAN_STEP_SIZE_METERS),
                    distance: current.distance + distance(next_state, &relevant_points),
                    prev: current_rc.clone(),
                    steps: current.steps + 1,
                })
            }
        }

        let all_points = points.get_all_points();
        println!(
            "{} points, final path cost: {}, evaluated {} paths in {}ms",
            all_points.len(),
            best_path.distance,
            total_paths,
            started.elapsed().as_millis(),
        );
        let final_path = reconstruct_path(best_path);
        draw_map_debug(&all_points, &final_path);
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
