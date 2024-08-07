mod colours;
mod comms;
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
use comms::{start_request_loop, CommsState, CONNECTED_TIMEOUT};
use components::{change_command_from_keys, driver_display, map_display, picture_taker, state_selector};
use eframe::egui::{self, Color32, Pos2, RichText};
use messages::command::CommandMode;
use std::{
    env, net::{IpAddr, SocketAddr}, str::FromStr, sync::{Arc, Mutex}, time::{Duration, Instant}
};

impl ToString for CommandMode {
    fn to_string(&self) -> String {
        match *self {
            CommandMode::StateOff => "Disabled",
            CommandMode::StateAuto => "  Auto  ",
            CommandMode::StateManual => " Manual ",
        }
        .to_string()
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

    start_request_loop(Arc::clone(&state_main));

    let mut mode = CommandMode::StateOff;
    let mut map_center = Pos2 { x: 0., y: 0. };
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut ip_str = match args.first() {
        None => "192.168.155.23".to_owned(),
        Some(ip_str) => ip_str.clone(),
    };
    let mut is_connected = false;
    let mut last_time = Instant::now();
    let mut delta_time = Duration::from_millis(16);
    let mut client_fps_avg = 0.0;
    let mut points_per_second = 0.0;

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Car IP Address");
                ui.text_edit_singleline(&mut ip_str);
            });
            if is_connected {
                ui.label(RichText::new("Connected").color(Color32::LIGHT_GREEN));
            } else {
                ui.label(RichText::new("Disconnected").color(Color32::RED));
            }
            state_selector(ui, &mut mode);

            {
                let mut state = state_main.lock().unwrap();
                state.command_to_send.state = mode as i32;
                let ip_addr_result = IpAddr::from_str(&ip_str);
                if let Ok(ip_addr) = ip_addr_result {
                    state.ip = SocketAddr::new(ip_addr, 3141);
                }

                change_command_from_keys(ui, delta_time, &mut state.command_to_send, &mut mode);
                driver_display(
                    ui,
                    &state.command_to_send,
                    &state.last_recieved_diagnostic.diagnostic.clone().unwrap_or_default(),
                );
                map_display(ui, &state.map, &state.last_recieved_diagnostic.path.clone().unwrap_or_default(), &mut map_center);

                is_connected = state.last_message_at.elapsed() < CONNECTED_TIMEOUT;

                let latency_ms = state.last_latency.as_secs_f64() * 1000.;
                ui.label(format!("Latency: {latency_ms:.1}ms"));
                if let Some(diag) = state.last_recieved_diagnostic.diagnostic.clone() {
                    let fps_avg = diag.framerate_avg;
                    let fps_low = diag.framerate_90;
                    ui.label(format!("Car fps avg: {:.2}", fps_avg));
                    ui.label(format!("Car fps low: {:.2}", fps_low));
                } else {
                    ui.label(format!("No diagnostic recieved"));
                    ui.label(format!("-"));
                }
                let alpha = 0.03;
                client_fps_avg = client_fps_avg * (1.0-alpha) + (1.0/delta_time.as_secs_f32()) * alpha;
                ui.label(format!("Client fps: {:.1}", client_fps_avg));
                let points_added = state.last_recieved_diagnostic.map_update.clone().map(|m| m.points_added.len()).unwrap_or_default() as f32;
                points_per_second = points_per_second * (1.0-alpha) + points_added * alpha;
                ui.label(format!("Points: {} ({:.1}/t)", state.map.len(), points_per_second));
                picture_taker(ui, &mut state.command_to_send, &ip_str);
            }
        });
        ctx.request_repaint();
        delta_time = last_time.elapsed();
        last_time = Instant::now();
    })
}
