use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use prost::Message;

use crate::{
    logging::Logger,
    messages::{self, command::CommandMode},
};

// Something to recive commands from
pub trait Commander {
    fn get_latest_message(&self) -> messages::command::DriveCommand;
}

// Manages communication over TCP with one or more client GUIs
pub struct NetworkCommsData {
    last_recieved: messages::command::DriveCommand,
    recived_at: Instant,
    to_send: messages::diagnostic::FullDiagnostic,
}

pub struct NetworkComms {
    data: Arc<Mutex<NetworkCommsData>>
}

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let data = NetworkCommsData {
            last_recieved: messages::command::DriveCommand::default(),
            recived_at: Instant::now() - COMMAND_TIMEOUT,
            to_send: messages::diagnostic::FullDiagnostic::default(),
        };
        let instance = NetworkComms { data: Arc::new(Mutex::new(data)) };
        instance.start_accept_loop();
        instance
    }

    // Starts a thread which accepts incoming connections
    pub fn start_accept_loop(&self) {
        let listener = TcpListener::bind("0.0.0.0:3141").unwrap();
        // not sure if the arc clone is nessisary twice, but it was giving errors without
        let data_copy = Arc::clone(&self.data);
        thread::spawn(move || {
            for stream in listener.incoming() {
                Self::start_recv_loop(
                    stream.unwrap(),
                    Arc::clone(&data_copy),
                );
            }
        });
    }

    // Starts a thread which waits for incoming commands
    // and then responds with the most recent diagnostics
    pub fn start_recv_loop(mut stream: TcpStream, data_mutex: Arc<Mutex<NetworkCommsData>>) {
        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                // Wait for and recieve command
                stream.read(&mut buf).unwrap();
                let mut data = data_mutex.lock().unwrap();
                data.last_recieved.clear();
                data.last_recieved.merge_length_delimited(&buf[..]).unwrap();

                // Send diagnostic update
                let to_send_buf = data.to_send.encode_length_delimited_to_vec();
                reset_map(&mut data.to_send);

                let sent = stream.write(&to_send_buf);
                if let Err(err) = sent {
                    println!("Connection write failed: '{}', closing this recieving thread", err);
                    return;
                }
                
                // Refresh safety timeout
                data.recived_at = Instant::now();
            }
        });
    }
}

impl Logger for NetworkComms {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic) {
        puffin::profile_function!();

        let mut data = self.data.lock().unwrap();
        accumulate_diagnostic_map(&mut data.to_send, message);
    }
}

fn accumulate_diagnostic_map(
    existing: &mut messages::diagnostic::FullDiagnostic, new: &messages::diagnostic::FullDiagnostic,
) {
    // Replace the diagnostic and path with most recent
    existing.diagnostic = new.diagnostic.clone();
    existing.path = new.path.clone();
    // Accumulate map updates
    existing.map_update = match (&mut existing.map_update, &new.map_update) {
        (None, _) => new.map_update.clone(),
        (Some(old_map), Some(new_map)) => {
            accumulate_map_update(old_map, new_map);
            existing.map_update.clone() // TODO: avoid clone
        }
        (Some(_), None) => existing.map_update.clone(),
    };
}

fn accumulate_map_update(
    existing_map_update: &mut messages::path::MapUpdate, _new_map_update: &messages::path::MapUpdate,
) {
    let mut new_map = _new_map_update.clone();

    // Handle remove ids that are for points that have not yet been sent
    let mut redundant_ids = Vec::new();
    for id in new_map.removed_ids.iter() {
        if let Some(index) = existing_map_update
            .points_added
            .iter()
            .position(|point| point.id == *id)
        {
            existing_map_update.points_added.remove(index);
            redundant_ids.push(*id);
        }
    }
    new_map.removed_ids.retain_mut(|p| !redundant_ids.contains(p));

    // Combine remaining adds and removes
    existing_map_update
        .points_added
        .append(&mut new_map.points_added.clone());
    existing_map_update.removed_ids.append(&mut new_map.removed_ids.clone());
}

fn reset_map(target: &mut messages::diagnostic::FullDiagnostic) {
    target.map_update = None;
}

const COMMAND_TIMEOUT: Duration = Duration::from_millis(300);

impl Commander for NetworkComms {
    fn get_latest_message(&self) -> messages::command::DriveCommand {
        puffin::profile_function!();

        let data = self.data.lock().unwrap();

        if data.recived_at.elapsed() > COMMAND_TIMEOUT {
            return messages::command::DriveCommand {
                state: CommandMode::StateOff as i32,
                throttle: 0.,
                turn: 0.,
                images_blue: data.last_recieved.images_blue,
                images_yellow: data.last_recieved.images_yellow,
                images_frame: data.last_recieved.images_frame,
            };
        }

        data.last_recieved.clone()
    }
}
