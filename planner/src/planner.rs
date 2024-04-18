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

use std::cmp::Ord;
use std::rc::Rc;
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::config::plan::{MAX_CURVATURE, PLAN_STEPS, PLAN_STEP_SIZE_METERS};
use crate::display::draw_map_debug;
use crate::points::{Point, PointMap, Pos};
use crate::state::CarState;

mod distance_calculators {
    use crate::points::Point;

    use super::CarState;

    pub fn calculate_avoid_edge_weight_for_point(state: CarState, point: &Point) -> f64 {
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

    pub fn calculate_travel_direction_weight_for_point(state: CarState, point: &Point) -> f64 {
        // add weight for travelling the wrong angular direction around points
        // extra for arrow points and none for obstacle points
        0.0
    }

    pub fn calculate_curvature_weight(state: CarState) -> f64 {
        // add weighting to enourage taking smoother lines
        state.curvature.abs().powf(2.0) * 0.2
    }
}

// Calculates the distance/traversability weights used for pathfinding
fn distance(state: CarState, nearby_points: &Vec<&Point>) -> f64 {
    puffin::profile_function!();
    let mut total_weight = -PLAN_STEP_SIZE_METERS;

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
    puffin::profile_function!();

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

        let mut open_set = BinaryHeap::new();
        open_set.push(PathNodeData {
            state: start_state,
            distance: 0.0,
            prev: Rc::new(PathNode::End),
            steps: 0,
        });
        let mut total_paths = 0;
        while let Some(current) = open_set.pop() {
            total_paths += 1;
            let current_rc = Rc::new(PathNode::Node(current.clone()));

            if current.steps > PLAN_STEPS {
                let all_points = points.get_points_in_area(Pos { x: 0., y: 0. }, 999.0);
                println!(
                    "{} points, final path cost: {}, evaluated {} paths",
                    all_points.len(),
                    current.distance,
                    total_paths
                );
                let final_path = reconstruct_path(current);
                draw_map_debug(&all_points, &final_path);
                return final_path;
            }

            let next_drive_states = get_possible_next_states(current.state).into_iter();
            let relevant_points = points.get_points_in_area(current.state.pos, 0.5); // TODO: magic number
            let get_node_from_state = |state: CarState| PathNodeData {
                state: state.step_distance(PLAN_STEP_SIZE_METERS),
                distance: current.distance + distance(state, &relevant_points),
                prev: current_rc.clone(),
                steps: current.steps + 1,
            };
            let next_nodes = next_drive_states.map(get_node_from_state);
            open_set.extend(next_nodes);
        }

        println!("Didn't find any paths, should be impossible");
        let no_path = Path { points: Vec::new() };
        draw_map_debug(
            &points.get_points_in_area(Pos { x: 0., y: 0. }, 999.0),
            &no_path,
        );
        no_path
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
