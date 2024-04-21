use crate::messages;
use prost::Message;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub const CONNECTED_TIMEOUT: Duration = Duration::from_millis(100);

pub struct CommsState {
    pub command_to_send: messages::command::DriveCommand,
    pub last_recieved_diagnostic: messages::diagnostic::FullDiagnostic,
    pub last_latency: Duration,
    pub last_message_at: Instant,
    pub map: Vec<messages::path::MapPoint>,
    pub ip: SocketAddr,
}

impl Default for CommsState {
    fn default() -> Self {
        CommsState {
            command_to_send: messages::command::DriveCommand::default(),
            last_recieved_diagnostic: messages::diagnostic::FullDiagnostic::default(),
            last_latency: Duration::ZERO,
            last_message_at: Instant::now().checked_sub(CONNECTED_TIMEOUT).unwrap(),
            map: Vec::new(),
            ip: SocketAddr::from(([127, 0, 0, 1], 3141)),
        }
    }
}

fn wait_to_connect(state: Arc<Mutex<CommsState>>) -> TcpStream {
    let mut count = 0;
    loop {
        {
            let local_state = state.lock().unwrap();
            println!("Trying to connect");
            match TcpStream::connect(local_state.ip) {
                Ok(connection) => return connection,
                Err(e) => println!(
                    "Connection to {} failed {} times: '{}', retying in 1s",
                    local_state.ip,
                    count,
                    e.to_string()
                ),
            };
            count += 1;
        }
        thread::sleep(Duration::from_secs(1));
    }
}

fn update_map(map: &mut Vec<messages::path::MapPoint>, map_update: &messages::path::MapUpdate) {
    map.extend(map_update.points_added.clone());
    map.retain(|point| !map_update.removed_ids.contains(&point.id));
}

pub fn start_request_loop(state: Arc<Mutex<CommsState>>) {
    thread::spawn(move || {
        let mut connection = wait_to_connect(Arc::clone(&state));
        connection.set_nodelay(true).unwrap();

        let mut buf = [0; 100000];
        loop {
            {
                let mut local_state = state.lock().unwrap();

                let message_sent_at = Instant::now();
                let to_send = local_state.command_to_send.encode_length_delimited_to_vec();
                connection.write(&to_send).unwrap();

                let recieved_bytes = connection.read(&mut buf[..]).unwrap();
                local_state.last_latency = message_sent_at.elapsed();
                local_state.last_message_at = Instant::now();

                local_state.last_recieved_diagnostic.clear();
                let result = local_state
                    .last_recieved_diagnostic
                    .merge_length_delimited(&buf[..]);

                if result.is_err() {
                    println!("Could not decode messages {} bytes", recieved_bytes);
                    continue;
                }

                let map_update = local_state
                    .last_recieved_diagnostic
                    .map_update
                    .clone()
                    .unwrap_or_default();
                update_map(&mut local_state.map, &map_update);
            }
            std::thread::sleep(Duration::from_secs_f64(0.033));
        }
    });
}
