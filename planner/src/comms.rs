use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread, time::{Duration, Instant},
};

use prost::Message;

use crate::{logging::Logger, messages::{self, command::CommandMode}};

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
        let listener = TcpListener::bind("127.0.0.1:3141").unwrap();
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
        mut stream: TcpStream,
        recieved_mutex: Arc<Mutex<messages::command::DriveCommand>>,
        to_send_mutex: Arc<Mutex<messages::diagnostic::FullDiagnostic>>,
        time_mutex: Arc<Mutex<Instant>>,
    ) {
        thread::spawn(move || {
            let mut buf = [0; 2048];
            loop {
                // Scopes are to prevent deadlocks by not taking multiple locks at the same time
                {
                    // Wait for and recieve command
                    stream.read(&mut buf).unwrap();
                    let mut recieved = recieved_mutex.lock().unwrap();
                    recieved.merge_length_delimited(&buf[..]).unwrap();
                }

                {
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
        let mut to_send = self.to_send.lock().unwrap();
        accumulate_diagnostic_map(&mut to_send, message);
    }
}

fn accumulate_diagnostic_map(target: &mut messages::diagnostic::FullDiagnostic, new: &messages::diagnostic::FullDiagnostic) {
    // Replace the diagnostic and path with most recent
    target.diagnostic = new.diagnostic.clone();
    target.path = new.path.clone();
    // Accumulate map updates
    target.map_update = match (&mut target.map_update, &new.map_update) {
        (None, _) => new.map_update.clone(), 
        (Some(old_map), Some(new_map))  => {
            old_map.points_added.append(&mut new_map.points_added.clone());
            old_map.removed_ids.append(&mut new_map.removed_ids.clone());
            target.map_update.clone() // TODO: avoid clone
        },
        (Some(_), None) => target.map_update.clone(),
    };
}

fn reset_map(target: &mut messages::diagnostic::FullDiagnostic) {
    target.map_update = None;
}

const COMMAND_TIMEOUT: Duration = Duration::from_millis(100);

impl Commander for NetworkComms {
    fn get_latest_message(&self) -> messages::command::DriveCommand {
        {
            let last_recived_at_local = self.recived_at.lock().unwrap();

            if last_recived_at_local.elapsed() > COMMAND_TIMEOUT {
                return messages::command::DriveCommand {
                    state: CommandMode::StateOff as i32,
                    throttle: 0.,
                    turn: 0.
                };
            }
        }

        let last_recieved_local = self.last_recieved.lock().unwrap();
        last_recieved_local.clone()
    }
}
