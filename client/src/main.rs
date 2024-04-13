
// TcpListener, tokio, socket2

use std::{borrow::Borrow, io::{Read, Write}, net::{SocketAddr, TcpStream}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use eframe::egui;
use prost::Message;
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

enum CarMode {
    OFF,
    AUTO,
    MANUAL,
}

impl ToString for CarMode {
    fn to_string(&self) -> String {
        match *self {
            CarMode::OFF => "Disabled",
            CarMode::AUTO => "Auto",
            CarMode::MANUAL => "Manual"
        }.to_string()
    }
}

fn state_selector(ui: &mut egui::Ui, current_state: &mut CarMode) {
    ui.horizontal(|ui| {
        ui.label(current_state.to_string());
        if ui.button("Stop").clicked() {
            *current_state = CarMode::OFF;
        }
        if ui.button("Auto").clicked() {
            *current_state = CarMode::AUTO;
        }
        if ui.button("Manual").clicked() {
            *current_state = CarMode::MANUAL;
        }
    });
}

fn wait_to_connect() -> TcpStream {
    let car_addr = SocketAddr::from(([127, 0, 0, 1], 3141));
    loop {
        println!("Trying to connect");
        match TcpStream::connect(car_addr) {
            Ok(connection) => return connection,
            Err(e) => println!("Connection failed '{}', retying in 1s", e.to_string()),
        };
        thread::sleep(Duration::from_secs(1));
    }
}

#[derive(Default)]
struct State {
    command_to_send: messages::command::DriveCommand,
    last_recieved_diagnostic: messages::diagnostic::FullDiagnostic,
    last_latency: Duration,
}

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    

    let state_main = Arc::new(Mutex::new(State::default()));
    let state_comms = Arc::clone(&state_main);

    let mut connection = wait_to_connect();
    
    thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            {
                let mut local_state = state_comms.lock().unwrap();
                
                let message_sent_at = Instant::now();
                connection.write(&local_state.command_to_send.encode_length_delimited_to_vec()).unwrap();
                connection.read(&mut buf[..]).unwrap();
                local_state.last_latency = message_sent_at.elapsed();

                local_state.last_recieved_diagnostic.merge_length_delimited(&buf[..]).unwrap();
            }
            std::thread::sleep(Duration::from_secs_f64(0.033));
        }
    });

    // Our application state:
    let mut mode = CarMode::OFF;
    let mut ip = String::new();

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Car IP Address");
                ui.text_edit_singleline(&mut ip);
            });
            ui.heading("UTS DRC 24");
            state_selector(ui, &mut mode);

            {
                let state = state_main.lock().unwrap();
                let latency_ms = state.last_latency.as_secs_f64() * 1000.;
                ui.label(format!("Latency: {latency_ms}ms"));
            }
        });
        {
            let mut state = state_main.lock().unwrap();
            state.command_to_send = messages::command::DriveCommand {
                state: (*(mode.borrow().clone())) as i32,
                throttle: 0.,
                turn: 0.,
            };

        }
    })
}