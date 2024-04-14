use std::{net::{SocketAddr, TcpStream}, thread, time::{Duration, Instant}};

use crate::messages;

pub fn wait_to_connect() -> TcpStream {
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

pub const CONNECTED_TIMEOUT: Duration = Duration::from_millis(100);

pub struct CommsState {
    pub command_to_send: messages::command::DriveCommand,
    pub last_recieved_diagnostic: messages::diagnostic::FullDiagnostic,
    pub last_latency: Duration,
    pub last_message_at: Instant,
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
