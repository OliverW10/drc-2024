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

// TODO: could impl deref?
pub struct MapPointWithTime {
    pub inner: messages::path::MapPoint,
    at: Instant,
}

pub struct CommsState {
    pub command_to_send: messages::command::DriveCommand,
    pub last_recieved_diagnostic: messages::diagnostic::FullDiagnostic,
    pub last_latency: Duration,
    pub last_message_at: Instant,
    pub map: Vec<MapPointWithTime>,
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
            ip: SocketAddr::from(([0, 0, 0, 1], 3141)),
        }
    }
}

fn wait_to_connect(state: Arc<Mutex<CommsState>>) -> TcpStream {
    let mut count = 0;
    loop {
        println!("start of loop");
        let ip = {
            let local_state = state.lock().unwrap();
            local_state.ip.clone()
        };
        println!("Trying to connect");
        match TcpStream::connect_timeout(&ip, Duration::from_millis(500)) {
            Ok(connection) => {
                println!("Connected");
                return connection
            }
            Err(e) => println!(
                "Connection to {} failed {} times: '{}', retying in 1s",
                ip,
                count,
                e.to_string()
            ),
        };
        count += 1;
        thread::sleep(Duration::from_millis(500))
    }
}

const MAX_TIMEOUT: Duration = Duration::from_millis(300);
fn update_map(map: &mut Vec<MapPointWithTime>, map_update: &messages::path::MapUpdate) {
    let mut new_points = map_update
        .points_added
        .iter()
        .map(|p| MapPointWithTime {
            inner: p.clone(),
            at: Instant::now(),
        })
        .collect();
    map.append(&mut new_points);
    map.retain(|point| !(map_update.removed_ids.contains(&point.inner.id) || point.at.elapsed() > MAX_TIMEOUT));
}

pub fn start_request_loop(state: Arc<Mutex<CommsState>>) {
    thread::spawn(move || {
        let mut connection = wait_to_connect(Arc::clone(&state));
        connection.set_nodelay(true).unwrap();

        loop {
            {
                let mut local_state = state.lock().unwrap();

                let message_sent_at = Instant::now();
                send_request(&mut connection, &local_state);

                read_response(&mut connection, &mut local_state);
                local_state.last_latency = message_sent_at.elapsed();
                local_state.last_message_at = Instant::now();

                let map_update = local_state
                    .last_recieved_diagnostic
                    .map_update
                    .clone()
                    .unwrap_or_default();
                update_map(&mut local_state.map, &map_update);
            }
            std::thread::sleep(Duration::from_secs_f64(0.050));
        }
    });
}

fn send_request(connection: &mut TcpStream, state: &CommsState) {
    let to_send = state.command_to_send.encode_length_delimited_to_vec();
    connection.write(&to_send).unwrap();
}

const RECV_BUF_LEN: usize = 100000;

fn read_response(connection: &mut TcpStream, state: &mut CommsState) -> bool {
    let mut buf = [0; RECV_BUF_LEN];

    let mut total_recieved_bytes = connection.read(&mut buf[..]).unwrap();

    let length_delimiter = prost::decode_length_delimiter(&buf[..]).expect("Could not decode message length");
    // length delimiter is encoded as LEB128 which contains 7 bits per byte
    // https://en.wikipedia.org/wiki/LEB128
    let length_length = ((length_delimiter as f32).log2() / 7.0).ceil() as usize;
    let total_expected_length = length_length + length_delimiter;
    if total_expected_length > RECV_BUF_LEN {
        println!("expecting too many bytes {}", total_expected_length);
        return false;
    }

    // Message may be too big to fit in a single tcp packet so may be split into multiple reads
    let mut packets = 1;
    while total_recieved_bytes < total_expected_length {
        let recieved_bytes = connection.read(&mut buf[total_recieved_bytes..]).unwrap();
        total_recieved_bytes += recieved_bytes;
        packets += 1;
    }
    if total_expected_length > total_expected_length {
        println!("had very much entirely too many bytes");
        return false;
    }
    println!("expected: {:?}, actual: {} in {}", total_expected_length, total_recieved_bytes, packets);

    state.last_recieved_diagnostic.clear();
    let decode_result = state.last_recieved_diagnostic.merge_length_delimited(&buf[..]);

    if let Err(e) = decode_result {    
        println!("{e}");
        return false;
    }

    return true;
}