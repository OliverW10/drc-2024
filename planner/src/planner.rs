
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

use std::collections::BinaryHeap;
// use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq)]
pub struct DriveState {
    pos: Pos,
    angle: f64, // angle of the car
    curvature: f64, // turn angle
    speed: f64
}

impl DriveState {
    fn step(&self, time: f64) -> DriveState {
        let dist = time * self.speed;
        if self.curvature.abs() < 1e-3 {
            return DriveState
                {
                    pos: Pos {
                        x: self.pos.x + dist * self.angle.cos(), // TODO: make who method single case
                        y: self.pos.y + dist * self.angle.sin(),
                    },
                    angle: self.angle,
                    curvature: self.curvature,
                    speed: self.speed
                }
        }
        let radius = 1.0 / self.curvature;
        DriveState {
            pos: Pos {
                x: 1.0,
                y: dist.sin() * radius,
            },
            angle: self.angle + self.curvature * dist,
            curvature: self.curvature,
            speed: self.speed,
        }
    }
}

fn calculate_avoid_edge_weight_for_point(state: DriveState, point: Point) -> f64{
    // add weight for being close to the point
    let max_weight = 4.0;
    let start_dist = 0.4;
    let edge_dist = state.pos.dist(point.pos);

    // goes from max_weight when at the edge to 0 when at start_dist away from edge
    let weighting = (start_dist - edge_dist) / start_dist * max_weight;
    if weighting >= 0.0 { weighting  } else { 0.0 }
}

fn calculate_travel_direction_weight_for_point(state: DriveState, point: Point) -> f64{
    // add weight for travelling the wrong angular direction around points
    0.0
}

fn calculate_curvature_weight(state: DriveState) -> f64{
    // add weight for turning
    state.curvature * 5.0
}

fn calculate_distance_for_point(state: DriveState, point: Point) -> f64{
    let edge_weight = calculate_avoid_edge_weight_for_point(state, point);
    let direction_weight = calculate_travel_direction_weight_for_point(state, point);

    edge_weight + direction_weight
}

fn distance(start: Pos, state: DriveState, points: Vec<Point>) -> f64{
    let end = start + state.to_pos();
    let mut total_weight = start.dist(end) * calculate_curvature_weight(state);
    for point in points {
        total_weight += calculate_distance_for_point(state, point);
    }
    total_weight
}

fn get_next_states(state: DriveState) -> Vec<DriveState> {
    let mut output = Vec::new();
    // both per timestep
    let max_turn_diff = 0.2; // radians
    let max_speed_diff = 0.1; // percent
    // options per side, total is 2*x+1
    let turn_options = 2;
    let speed_options = 1;
    for turn_diff in -turn_options..turn_options+1 {
        for throttle in -speed_options..speed_options+1 {
            output.push(DriveState {
                pos: state.pos,
                curvature: state.curvature + max_turn_diff * (turn_diff / turn_options) as f64,
                speed: state.speed + max_speed_diff * (throttle / speed_options) as f64,
                angle: state.angle,
            }.step(0.1));
        }
    }
    output
}

fn pathfind(start_state: DriveState, points: PointMap) -> () {
    // https://doc.rust-lang.org/std/collections/binary_heap/index.html
    let open_set = BinaryHeap::new();
    open_set.push(start_state);
    get_next_states();
}