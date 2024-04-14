
// TcpListener, tokio, socket2
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
use std::{io::{Read, Write}, net::{SocketAddr, TcpStream}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use eframe::egui;
use messages::command::CommandMode;
use prost::Message;


impl ToString for CommandMode {
    fn to_string(&self) -> String {
        match *self {
            CommandMode::StateOff => "Disabled",
            CommandMode::StateAuto => "Auto",
            CommandMode::StateManual => "Manual"
        }.to_string()
    }
}


fn state_selector(ui: &mut egui::Ui, current_state: &mut CommandMode) {
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

fn wait_to_connect() -> TcpStream {
    let car_addr = SocketAddr::from(([127, 0, 0, 1], 3141));
    let mut count = 0;
    loop {
        println!("Trying to connect");
        match TcpStream::connect(car_addr) {
            Ok(connection) => return connection,
            Err(e) => println!("Connection failed {}: '{}', retying in 1s", count, e.to_string()),
        };
        count += 1;
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
    connection.set_nodelay(true).unwrap();
    
    thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            {
                let mut local_state = state_comms.lock().unwrap();
                
                let message_sent_at = Instant::now();
                let to_send = local_state.command_to_send.encode_length_delimited_to_vec();
                connection.write(&to_send).unwrap();
                let recieved_bytes = connection.read(&mut buf[..]).unwrap();
                local_state.last_latency = message_sent_at.elapsed();
                
                println!("recieved {} bytes, sent {} bytes", recieved_bytes, to_send.len());
                local_state.last_recieved_diagnostic.merge_length_delimited(&buf[..]).unwrap();
            }
            std::thread::sleep(Duration::from_secs_f64(0.033));
        }
    });

    // Our application state:
    let mut mode = CommandMode::StateOff;
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
                state: mode.clone() as i32,
                throttle: 0.,
                turn: 0.2,
            };

        }
    })
}
