
// TcpListener, tokio, socket2

use std::{borrow::BorrowMut, io::{Read, Write}, net::{SocketAddr, TcpStream}, sync::{Arc, Mutex}, thread, time::{Duration, Instant, SystemTime}};
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

enum State{
    OFF,
    AUTO,
    MANUAL,
}

impl ToString for State {
    fn to_string(&self) -> String {
        match *self {
            State::OFF => "Disabled",
            State::AUTO => "Auto",
            State::MANUAL => "Manual"
        }.to_string()
    }
}

fn state_selector(ui: &mut egui::Ui, current_state: &mut State) {
    ui.horizontal(|ui| {
        ui.label(current_state.to_string());
        if ui.button("Stop").clicked() {
            *current_state = State::OFF;
        }
        if ui.button("Auto").clicked() {
            *current_state = State::AUTO;
        }
        if ui.button("Manual").clicked() {
            *current_state = State::MANUAL;
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


fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    

    let command_main = Arc::new(Mutex::new(messages::command::DriveCommand::default()));
    let reading_main = Arc::new(Mutex::new(messages::diagnostic::FullDiagnostic::default()));

    let command_comms = Arc::clone(&command_main);
    let reading_comms = Arc::clone(&reading_main);
    let mut latency = Arc::new(Mutex::new(Duration::from_secs_f32(1.)));

    let mut connection = wait_to_connect();
    
    thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            {
                let command = command_comms.lock().unwrap();
                let mut diagnostic = reading_comms.lock().unwrap();
                let mut latency = latency.lock().unwrap();
                
                let message_sent_at = Instant::now();
                connection.write(&command.encode_length_delimited_to_vec()).unwrap();
                connection.read(&mut buf[..]).unwrap();
                latency.borrow_mut() = message_sent_at.elapsed();

                diagnostic.merge_length_delimited(&buf[..]).unwrap();
            }
            std::thread::sleep(Duration::from_secs_f64(0.33));
        }
    });

    // Our application state:
    let mut state = State::OFF;
    let mut ip = String::new();

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Car IP Address");
                ui.text_edit_singleline(&mut ip);
            });
            ui.heading("UTS DRC 24");
            state_selector(ui, &mut state);

            let mut x = command_main.lock().unwrap();
            x.throttle = 0.2;
            x.throttle = 0.2;
            
            let latency_ms = latency.as_secs_f64() * 1000.;
            ui.label(format!("Latency: {latency_ms}ms"));
        });
    })
}