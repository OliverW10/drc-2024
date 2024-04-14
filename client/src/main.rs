mod colours;
mod components;
mod messages {
    pub mod path {
        include!(concat!(env!("OUT_DIR"), "/messages.path.rs"));
    }
    pub mod diagnostic {
        include!(concat!(env!("OUT_DIR"), "/messages.diagnostic.rs"));
    }
    pub mod command {
        include!(concat!(env!("OUT_DIR"), "/messages.commands.rs"));
    }
}
use eframe::egui::{self, Key, Pos2, Rect, RichText, Stroke, Vec2};
use messages::{
    command::CommandMode,
    diagnostic::FullDiagnostic,
};
use prost::Message;
use rand::random;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

impl ToString for CommandMode {
    fn to_string(&self) -> String {
        match *self {
            CommandMode::StateOff =>  "Disabled",
            CommandMode::StateAuto => "  Auto  ",
            CommandMode::StateManual=>" Manual ",
        }
        .to_string()
    }
}

fn state_selector(ui: &mut egui::Ui, current_state: &mut CommandMode) {
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
fn map(ui: &mut egui::Ui, map: &FullDiagnostic) {
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

fn change_command_from_keys(ui: &mut egui::Ui, dt: Duration, command: &mut messages::command::DriveCommand) {

    let keys = ui.input(|i| i.keys_down.clone());
    let is_left = keys.contains(&Key::ArrowLeft) || keys.contains(&Key::A);
    let is_right = keys.contains(&Key::ArrowRight) || keys.contains(&Key::D);
    let is_up = keys.contains(&Key::ArrowUp) || keys.contains(&Key::W);
    let is_down = keys.contains(&Key::ArrowDown) || keys.contains(&Key::S);

    command.throttle = change_input(dt, command.throttle, is_up, is_down, ACCEL, MAX_SPEED, SPEED_DECAY);
    command.turn = change_input(dt, command.turn, is_right, is_left, TURN_RATE, MAX_TURN, TURN_DECAY);
}

fn driver_display(
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

fn wait_to_connect() -> TcpStream {
    let car_addr = SocketAddr::from(([127, 0, 0, 1], 3141));
    let mut count = 0;
    loop {
        println!("Trying to connect");
        match TcpStream::connect(car_addr) {
            Ok(connection) => return connection,
            Err(e) => println!(
                "Connection failed {}: '{}', retying in 1s",
                count,
                e.to_string()
            ),
        };
        count += 1;
        thread::sleep(Duration::from_secs(1));
    }
}

const CONNECTED_TIMEOUT: Duration = Duration::from_millis(100);

struct CommsState {
    command_to_send: messages::command::DriveCommand,
    last_recieved_diagnostic: messages::diagnostic::FullDiagnostic,
    last_latency: Duration,
    last_message_at: Instant,
}

impl Default for CommsState {
    fn default() -> Self {
        CommsState {
            command_to_send: messages::command::DriveCommand::default(),
            last_recieved_diagnostic: messages::diagnostic::FullDiagnostic::default(),
            last_latency: Duration::ZERO,
            last_message_at: Instant::now().checked_sub(CONNECTED_TIMEOUT).unwrap(),
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };

    let state_main = Arc::new(Mutex::new(CommsState {
        last_latency: Duration::from_secs(u64::MAX),
        ..Default::default()
    }));
    let state_comms = Arc::clone(&state_main);

    thread::spawn(move || {
        let mut connection = wait_to_connect();
        connection.set_nodelay(true).unwrap();

        let mut buf = [0; 4096];
        loop {
            {
                let mut local_state = state_comms.lock().unwrap();

                let message_sent_at = Instant::now();
                let to_send = local_state.command_to_send.encode_length_delimited_to_vec();
                connection.write(&to_send).unwrap();
                let recieved_bytes = connection.read(&mut buf[..]).unwrap();
                local_state.last_latency = message_sent_at.elapsed();
                local_state.last_message_at = Instant::now();

                println!(
                    "recieved {} bytes, sent {} bytes",
                    recieved_bytes,
                    to_send.len()
                );
                local_state
                    .last_recieved_diagnostic
                    .merge_length_delimited(&buf[..])
                    .unwrap();
            }
            std::thread::sleep(Duration::from_secs_f64(0.033));
        }
    });

    let mut mode = CommandMode::StateOff;
    let mut ip = String::new();
    let mut is_connected = false;
    let mut last_time = Instant::now();
    let mut delta_time = Duration::from_millis(16);

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Car IP Address");
                ui.text_edit_singleline(&mut ip);
            });
            if is_connected {
                ui.label(RichText::new("Connected").color(colours::CONNECTED_TEXT));
            } else {
                ui.label(RichText::new("Disconnected").color(colours::NOT_CONNECTED_TEXT));
            }
            ui.heading("UTS DRC 24");
            state_selector(ui, &mut mode);
            {
                let mut state = state_main.lock().unwrap();
                state.command_to_send.state = mode as i32;
                change_command_from_keys(ui, delta_time, &mut state.command_to_send);
                driver_display(ui, &state.command_to_send);
                map(ui, &state.last_recieved_diagnostic);

                is_connected = state.last_message_at.elapsed() < CONNECTED_TIMEOUT;

                let latency_ms = state.last_latency.as_secs_f64() * 1000.;
                ui.label(format!("Latency: {latency_ms}ms"));
            }
        });
        ctx.request_repaint();
        delta_time = last_time.elapsed();
        last_time = Instant::now();
    })
}
