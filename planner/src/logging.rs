use crate::messages::{
    path::{Path, Points},
    diagnostic::Diagnostic
};


// Something to log complex diagnostic and debgging info such as sensor readings, and planned actions
pub trait Logger {
    fn send(&self, path: &Path, new_points: &Points, diagnostic: &Diagnostic);
}

pub struct FileWriter {

}

impl Logger for FileWriter {
    fn send(&self, path: &Path, new_points: &Points, diagnostic: &Diagnostic) {}
}


// Simple struct to multicast logging
pub struct AggregateLogger {
    loggers: Vec<Box<dyn Logger>>
}

impl AggregateLogger {
    pub fn new(loggers: Vec<Box<dyn Logger>>) -> AggregateLogger {
        AggregateLogger { loggers: loggers }
    }
}

impl Logger for AggregateLogger {
    fn send(&self, path: &Path, new_points: &Points, diagnostic: &Diagnostic) {
        for logger in self.loggers.iter() {
            logger.send(path, new_points, diagnostic);
        }
    }
}
