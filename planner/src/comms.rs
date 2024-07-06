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
pub struct NetworkComms {
    last_recieved: Arc<Mutex<messages::command::DriveCommand>>,
    recived_at: Arc<Mutex<Instant>>,
    to_send: Arc<Mutex<messages::diagnostic::FullDiagnostic>>,
}

impl NetworkComms {
    pub fn new() -> NetworkComms {
        let instance = NetworkComms {
            last_recieved: Arc::new(Mutex::new(messages::command::DriveCommand::default())),
            recived_at: Arc::new(Mutex::new(Instant::now() - COMMAND_TIMEOUT)),
            to_send: Arc::new(Mutex::new(messages::diagnostic::FullDiagnostic::default())),
        };
        instance.start_accept_loop();
        instance
    }

    // Starts a thread which accepts incoming connections
    pub fn start_accept_loop(&self) {
        let listener = TcpListener::bind("0.0.0.0:3141").unwrap();
        // not sure if the arc clone is nessisary twice, but it was giving errors without
        let last_recieved = Arc::clone(&self.last_recieved);
        let to_send = Arc::clone(&self.to_send);
        let recived_at = Arc::clone(&self.recived_at);
        thread::spawn(move || {
            for stream in listener.incoming() {
                Self::start_recv_loop(
                    stream.unwrap(),
                    Arc::clone(&last_recieved),
                    Arc::clone(&to_send),
                    Arc::clone(&recived_at),
                );
            }
        });
    }

    // Starts a thread which waits for incoming commands
    // and then responds with the most recent diagnostics
    pub fn start_recv_loop(
        mut stream: TcpStream, recieved_mutex: Arc<Mutex<messages::command::DriveCommand>>,
        to_send_mutex: Arc<Mutex<messages::diagnostic::FullDiagnostic>>, time_mutex: Arc<Mutex<Instant>>,
    ) {
        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                // Scopes are to prevent deadlocks by not taking multiple locks at the same time
                {
                    // Wait for and recieve command
                    stream.read(&mut buf).unwrap();
                    let mut recieved = recieved_mutex.lock().unwrap();
                    recieved.clear();
                    recieved.merge_length_delimited(&buf[..]).unwrap();
                }

                {
                    puffin::profile_scope!("send msg");

                    // Send diagnostic update
                    let mut to_send = to_send_mutex.lock().unwrap();
                    let to_send_buf = to_send.encode_length_delimited_to_vec();
                    let sent = stream.write(&to_send_buf);

                    if let Err(err) = sent {
                        println!("Connection write failed: '{}', closing this recieving thread", err);
                        break;
                    }
                    reset_map(&mut to_send);
                }

                {
                    // Refresh safety timeout
                    let mut timestamp = time_mutex.lock().unwrap();
                    *timestamp = Instant::now();
                }
            }
        });
    }
}

impl Logger for NetworkComms {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic) {
        puffin::profile_function!();

        let mut to_send = self.to_send.lock().unwrap();
        accumulate_diagnostic_map(&mut to_send, message);
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

        {
            let last_recived_at_local = self.recived_at.lock().unwrap();

            if last_recived_at_local.elapsed() > COMMAND_TIMEOUT {
                return messages::command::DriveCommand {
                    state: CommandMode::StateOff as i32,
                    throttle: 0.,
                    turn: 0.,
                };
            }
        }

        let last_recieved_local = self.last_recieved.lock().unwrap();
        last_recieved_local.clone()
    }
}
