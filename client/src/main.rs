
// TcpListener, tokio, socket2

use std::time::SystemTime;

use eframe::egui;

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

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]).,
        ..Default::default()
    };

    // Our application state:
    let mut state = State::OFF;
    let mut last_msg_at = SystemTime::now();
    let mut ip = String::new();

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Car IP Address");
                ui.text_edit_singleline(&mut ip);
            });
            ui.heading("UTS DRC 24");
            state_selector(ui, &mut state);
            
            let since_last_msg = SystemTime::now().duration_since(last_msg_at).unwrap().as_secs_f64();
            let latency_ms = since_last_msg * 1000.;
            let is_connected = latency_ms < 100.;
            ui.label(format!("Connected: {is_connected} latency: {latency_ms}ms"));
        });
    })
}