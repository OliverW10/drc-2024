use std::{iter::zip, time::Duration};

use eframe::egui::{self, Color32, Key, Pos2, Rect, Stroke, Vec2};

use crate::{colours::{self, point_colour}, messages::{self, command::CommandMode, path::PointType}};


pub fn state_selector(ui: &mut egui::Ui, current_mode: &mut CommandMode) {
    // TODO: radio buttons?
    ui.horizontal(|ui| {
        ui.label(current_mode.to_string());
        if ui.button("Stop").clicked() {
            *current_mode = CommandMode::StateOff;
        }
        if ui.button("Auto").clicked() {
            *current_mode = CommandMode::StateAuto;
        }
        if ui.button("Manual").clicked() {
            *current_mode = CommandMode::StateManual;
        }
    });
}

const DRIVER_RECT: Rect = Rect {
    min: Pos2 { x: 400., y: 5. },
    max: Pos2 { x: 600., y: 205. },
};

const MAP_RECT: Rect = Rect {
    min: Pos2 { x: 5., y: 200. },
    max: Pos2 { x: 405., y: 600. },
};

// takes a pos thats -1 to 1 and puts it ceneterd in the rect
fn in_rect(p: Pos2, r: Rect) -> Pos2 {
    // lerp_inside takes a 0-1 but we get a -1-1
    let vec = Vec2 {
        x: p.x / 2. + 0.5,
        y: p.y / 2. + 0.5,
    };
    r.lerp_inside(vec)
}

pub fn map_display(ui: &mut egui::Ui, map: &Vec<messages::path::MapPoint>, path: &messages::path::Path) {
    let map_center = Pos2 { x: 0., y: 0. };
    let map_scale = 1./4.; // 4x4 meter map
    let paint = ui.painter().with_clip_rect(MAP_RECT);
    paint.rect_filled(MAP_RECT, 0., colours::SHADE);

    for point in map {
        let pos = Pos2 { x: point.x, y: point.y };
        let point_type = PointType::try_from(point.point_type).unwrap();
        paint.circle(in_rect(pos*map_scale, MAP_RECT), 2., point_colour(&point_type), Stroke::NONE);
    }
    if path.points.len() > 0 {
        for (prev, next) in zip(&path.points[..], &path.points[1..]) {
            let a = in_rect(Pos2 {x: prev.x, y: prev.y} * map_scale, MAP_RECT);
            let b = in_rect(Pos2 {x: next.x, y: next.y } * map_scale, MAP_RECT);
            paint.line_segment([a, b], Stroke::new(1., Color32::WHITE));
        }
    }
}

const MAX_SPEED: f32 = 0.5;
const MAX_TURN: f32 = 3.;

const ACCEL: f32 = 1.;
const TURN_RATE: f32 = 10.;

const SPEED_DECAY: f32 = 1.;
const TURN_DECAY: f32 = 3.;

fn change_input(dt: Duration, last: f32, is_positive: bool, is_negative: bool, change_from_input: f32, max_output: f32, decay_rate: f32) -> f32 {
    let input = (is_positive as i32 - is_negative as i32) as f32;
    if input == 0. {
        let decay = decay_rate * dt.as_secs_f32();
        (0.0 as f32).clamp(last - decay, last + decay) // move last towards 0 by decay
    } else {
        let change = input * change_from_input * dt.as_secs_f32();
        (last + change).clamp(-max_output, max_output)
    }
}

pub fn change_command_from_keys(ui: &mut egui::Ui, dt: Duration, command: &mut messages::command::DriveCommand) {

    let keys = ui.input(|i| i.keys_down.clone());
    let is_left = keys.contains(&Key::ArrowLeft) || keys.contains(&Key::A);
    let is_right = keys.contains(&Key::ArrowRight) || keys.contains(&Key::D);
    let is_up = keys.contains(&Key::ArrowUp) || keys.contains(&Key::W);
    let is_down = keys.contains(&Key::ArrowDown) || keys.contains(&Key::S);

    command.throttle = change_input(dt, command.throttle, is_up, is_down, ACCEL, MAX_SPEED, SPEED_DECAY);
    command.turn = change_input(dt, command.turn, is_right, is_left, TURN_RATE, MAX_TURN, TURN_DECAY);
}

pub fn driver_display(
    ui: &mut egui::Ui,
    last_command: &messages::command::DriveCommand,
    actual_driven: &messages::diagnostic::Diagnostic,
) {

    let paint = ui.painter().with_clip_rect(DRIVER_RECT);
    
    paint.rect_filled(DRIVER_RECT, 0., colours::SHADE);

    let actual_pos = Pos2 {
        x: actual_driven.actual_turn / MAX_TURN,
        y: -actual_driven.actual_speed / MAX_SPEED,
    };
    paint.circle(
        in_rect(actual_pos, DRIVER_RECT),
        10.,
        colours::DRIVE_ACTUAL_BALL,
        Stroke::NONE,
    );

    let indicator_pos = Pos2 {
        x: last_command.turn / MAX_TURN,
        y: -last_command.throttle / MAX_SPEED,
    };
    paint.circle(
        in_rect(indicator_pos, DRIVER_RECT),
        10.,
        colours::DRIVE_COMMAND_BALL,
        Stroke::NONE,
    );
}
