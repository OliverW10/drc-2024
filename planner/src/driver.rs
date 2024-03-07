use crate::messages::path::SimpleDrive;

pub trait IDriver {
    fn drive(&self, command: SimpleDrive);
}

pub struct NetworkDriver {}

impl NetworkDriver {}

impl IDriver for NetworkDriver {
    fn drive(&self, command: SimpleDrive) {}
}

pub struct SerialDriver {}

impl SerialDriver {
    pub fn new() -> SerialDriver {
        SerialDriver {}
    }
}

impl IDriver for SerialDriver {
    fn drive(&self, command: SimpleDrive) {}
}
