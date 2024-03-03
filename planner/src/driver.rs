use crate::messages::path::SimpleDrive;

pub trait IDriver {
    fn drive(command: SimpleDrive);
}

pub struct NetworkDriver {}

impl NetworkDriver {}

impl IDriver for NetworkDriver {
    fn drive(command: SimpleDrive) {}
}

pub struct SerialDriver {}

impl SerialDriver {}

impl IDriver for SerialDriver {
    fn drive(command: SimpleDrive) {}
}
