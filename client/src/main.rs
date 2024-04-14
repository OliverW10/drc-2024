mod colours;
mod components;
mod comms;
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
use comms::{wait_to_connect, CommsState, CONNECTED_TIMEOUT};
use components::{change_command_from_keys, driver_display, map, state_selector};
use eframe::egui::{self, RichText};
use messages::command::CommandMode;
use prost::Message;
use std::{
    io::{Read, Write}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}
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
