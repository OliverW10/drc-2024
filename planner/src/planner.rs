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

use crate::config::plan::{PLAN_STEPS, PLAN_STEP_SIZE_METERS};
use crate::display::draw_map_debug;
use crate::points::{Point, PointMap, Pos};
use crate::state::DriveState;

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
        state.curvature.abs().powf(2.0) * 0.1
    }
}

// calculates the distance/traversability map used for pathfinding
fn distance(state: DriveState, nearby_points: &Vec<&Point>) -> f64 {
    let mut total_weight = -0.1;

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

const MAX_CURVATURE: f64 = 1.;
// const MAX_CURVATURE: f64 = 1.0 / 0.3;

pub fn get_possible_next_states(state: DriveState) -> Vec<DriveState> {
    let mut output = Vec::new();
    let turn_options = 3; // per side
    for new_turn_index in -turn_options..turn_options + 1 {
        let new_curvature = MAX_CURVATURE * (new_turn_index as f64 / turn_options as f64);
        let new_drive_state = DriveState {
            curvature: new_curvature,
            ..state
        };
        output.push(new_drive_state.step_distance(PLAN_STEP_SIZE_METERS));
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

impl Planner {
    pub fn new() -> Planner {
        Planner {}
    }

    pub fn find_path(&self, start_state: DriveState, points: &impl PointMap) -> Path {
        puffin::profile_function!();

        // https://doc.rust-lang.org/std/collections/binary_heap/index.html
        let mut open_set = BinaryHeap::new();
        open_set.push(PathNodeData {
            state: start_state,
            distance: 0.0,
            prev: Rc::new(PathNode::End),
            steps: 0,
        });
        while let Some(current) = open_set.pop() {
            let current_rc = Rc::new(PathNode::Node(current.clone()));

            if current.steps > PLAN_STEPS {
                let final_path = reconstruct_path(current);
                draw_map_debug(
                    &points.get_points_in_area(Pos { x: 0., y: 0. }, 999.0),
                    &final_path,
                )
                .unwrap();
                return final_path;
            }

            let next_drive_states = get_possible_next_states(current.state).into_iter();
            let relevant_points = points.get_points_in_area(current.state.pos, 0.5); // TODO: magic number
            let get_node_from_state = |state| PathNodeData {
                state: state,
                distance: current.distance + distance(state, &relevant_points),
                prev: current_rc.clone(),
                steps: current.steps + 1,
            };
            let next_nodes = next_drive_states.map(get_node_from_state);
            open_set.extend(next_nodes);
        }
        let no_path = Path { points: Vec::new() };
        draw_map_debug(
            &points.get_points_in_area(Pos { x: 0., y: 0. }, 999.0),
            &no_path,
        )
        .unwrap();
        no_path
    }
}

fn reconstruct_path(final_node: PathNodeData) -> Path {
    puffin::profile_function!();

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
