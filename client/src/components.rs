use std::time::Duration;

use eframe::egui::{self, Key, Pos2, Rect, Stroke, Vec2};

use crate::{colours, messages::{self, command::CommandMode, diagnostic::FullDiagnostic}};


pub fn state_selector(ui: &mut egui::Ui, current_state: &mut CommandMode) {
    // TODO: radio buttons?
    ui.horizontal(|ui| {
        ui.label(current_state.to_string());
        if ui.button("Stop").clicked() {
            *current_state = CommandMode::StateOff;
        }
        if ui.button("Auto").clicked() {
            *current_state = CommandMode::StateAuto;
        }
        if ui.button("Manual").clicked() {
            *current_state = CommandMode::StateManual;
        }
    });
}

const DRIVER_RECT: Rect = Rect {
    min: Pos2 { x: 400., y: 5. },
    max: Pos2 { x: 600., y: 205. },
};

const MAP_RECT: Rect = Rect {
    min: Pos2 { x: 5., y: 200. },
    max: Pos2 { x: 505., y: 700. },
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
pub fn map(ui: &mut egui::Ui, map: &FullDiagnostic) {
    let paint = ui.painter().with_clip_rect(MAP_RECT);
    paint.rect_filled(DRIVER_RECT, 0., colours::SHADE);
}

const MAX_SPEED: f32 = 0.5;
const MAX_TURN: f32 = 3.;

const ACCEL: f32 = 1.;
const TURN_RATE: f32 = 10.;

const SPEED_DECAY: f32 = 2.;
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
) -> messages::command::DriveCommand {

    let paint = ui.painter().with_clip_rect(DRIVER_RECT);

    
    let indicator_pos = Pos2 {
        x: last_command.turn / MAX_TURN,
        y: -last_command.throttle / MAX_SPEED,
    };
    paint.rect_filled(DRIVER_RECT, 0., colours::SHADE);
    paint.circle(
        in_rect(indicator_pos, DRIVER_RECT),
        10.,
        colours::DRIVE_BALL,
        Stroke::NONE,
    );

    messages::command::DriveCommand {
        state: last_command.state,
        throttle: 0.,
        turn: 0.2,
    }
}