use std::{iter::zip, process::Command, time::Duration};

use eframe::egui::{self, Align2, Color32, Key, Pos2, Rect, Stroke, Vec2};

use crate::{
    colours::point_colour,
    comms::MapPointWithTime,
    messages::{self, command::CommandMode, path::PointType},
};

pub fn state_selector(ui: &mut egui::Ui, current_mode: &mut CommandMode) {
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

pub fn picture_taker(ui: &mut egui::Ui, command: &mut messages::command::DriveCommand, ip: &str) {
    ui.horizontal(|ui| {
        if ui.button("Image").clicked() {
            command.images_frame += 1;
        }
        if ui.button("Blu Mask").clicked() {
            command.images_blue += 1;
        }
        if ui.button("Yel Mask").clicked() {
            command.images_yellow += 1;
        }
        if ui.button("Sync").clicked() {
            do_rsync(ip);
        }
    });
}

fn do_rsync(ip: &str) {
    // Command::new("rsync").arg(format!("pi@raspberrypi.local:~/drc-2024/planner/images/")).arg("images/").status().expect();
    let output = Command::new("rsync").arg(format!("../planner/images/")).arg("images/").output().expect("asdf");
    println!("rsynced: {}", std::str::from_utf8(&output.stdout).expect("msg"));
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

pub fn map_display(ui: &mut egui::Ui, map: &Vec<MapPointWithTime>, path: &messages::path::Path, map_center: &mut Pos2) {
    ui.horizontal(|ui| {
        ui.label(format!("x:{}, y:{}", map_center.x, map_center.y));
        if ui.button("Recenter").clicked() {
            let new_center = path.points.first().map(|pos| Pos2 {x: pos.x, y: pos.y}).unwrap_or(Pos2 { x: 0.0, y: 0.0});
            println!("Centering map on {},{}", new_center.x, new_center.y);
            map_center.x = new_center.x;
            map_center.y = new_center.y;
        }
    });
    let map_scale = 1. / 4.; // meters in each direction of origin
    let paint = ui.painter().with_clip_rect(MAP_RECT);
    paint.rect_filled(MAP_RECT, 0., Color32::DARK_GRAY);

    for point in map {
        let pos = Pos2 {
            x: point.inner.x - map_center.x,
            y: point.inner.y - map_center.y,
        };
        let point_type = PointType::try_from(point.inner.point_type).unwrap();
        let c = match point_type {
            PointType::ArrowLeft => "⟲",
            PointType::ArrowRight => "⟳",
            _ => "",
        };
        if c.len() == 0 {
            paint.circle(in_rect(pos * map_scale, MAP_RECT), 1., point_colour(&point_type), Stroke::NONE);
        } else {
            paint.text(in_rect(pos * map_scale, MAP_RECT), Align2::CENTER_CENTER, c, egui::FontId::monospace(12.0), point_colour(&point_type));
        }
    }

    if path.points.len() > 0 {
        for (prev, next) in zip(&path.points[..], &path.points[1..]) {
            let a = in_rect(Pos2 { x: prev.x - map_center.x, y: prev.y - map_center.y } * map_scale, MAP_RECT);
            let b = in_rect(Pos2 { x: next.x - map_center.x, y: next.y - map_center.y } * map_scale, MAP_RECT);
            paint.line_segment([a, b], Stroke::new(1., Color32::WHITE));
        }
    }
}

const MAX_SPEED: f32 = 0.75; //2.5;
const MAX_TURN: f32 = 1.4;

const ACCEL: f32 = 2.0;
const TURN_RATE: f32 = 5.;

const SPEED_DECAY: f32 = 1.0;
const TURN_DECAY: f32 = 3.0;

fn change_input(
    dt: Duration, last: f32, is_positive: bool, is_negative: bool, change_from_input: f32, max_output: f32,
    decay_rate: f32,
) -> f32 {
    let input = (is_positive as i32 - is_negative as i32) as f32;
    if input == 0. {
        let decay = decay_rate * dt.as_secs_f32();
        (0.0 as f32).clamp(last - decay, last + decay) // move last towards 0 by decay
    } else {
        let change = input * change_from_input * dt.as_secs_f32();
        (last + change).clamp(-max_output, max_output)
    }
}

pub fn change_command_from_keys(
    ui: &mut egui::Ui, dt: Duration, command: &mut messages::command::DriveCommand,
    mode: &mut messages::command::CommandMode,
) {
    let (keys, space) = ui.input(|i| (i.keys_down.clone(), i.key_released(Key::Space)));
    let is_left = keys.contains(&Key::ArrowLeft) || keys.contains(&Key::A);
    let is_right = keys.contains(&Key::ArrowRight) || keys.contains(&Key::D);
    let is_up = keys.contains(&Key::ArrowUp) || keys.contains(&Key::W);
    let is_down = keys.contains(&Key::ArrowDown) || keys.contains(&Key::S);

    command.throttle = change_input(dt, command.throttle, is_up, is_down, ACCEL, MAX_SPEED, SPEED_DECAY);
    command.turn = change_input(dt, command.turn, is_right, is_left, TURN_RATE, MAX_TURN, TURN_DECAY);

    if space {
        *mode = CommandMode::StateOff;
    }
}

pub fn driver_display(
    ui: &mut egui::Ui, last_command: &messages::command::DriveCommand, actual_driven: &messages::diagnostic::Diagnostic,
) {
    let paint = ui.painter().with_clip_rect(DRIVER_RECT);

    paint.rect_filled(DRIVER_RECT, 0., Color32::DARK_GRAY);
    let outline_col = match CommandMode::try_from(last_command.state) {
        Err(_) => Color32::RED,
        Ok(CommandMode::StateOff) => Color32::BLACK,
        Ok(CommandMode::StateAuto) => Color32::GREEN,
        Ok(CommandMode::StateManual) => Color32::GOLD,
    };
    paint.rect_stroke(DRIVER_RECT, 0., Stroke::new(3.0, outline_col));

    let actual_pos = Pos2 {
        x: actual_driven.actual_turn / MAX_TURN,
        y: -actual_driven.actual_speed / MAX_SPEED,
    };
    paint.circle(in_rect(actual_pos, DRIVER_RECT), 10., Color32::GRAY, Stroke::NONE);

    let indicator_pos = Pos2 {
        x: last_command.turn / MAX_TURN,
        y: -last_command.throttle / MAX_SPEED,
    };
    paint.circle(in_rect(indicator_pos, DRIVER_RECT), 10., Color32::WHITE, Stroke::NONE);
}
