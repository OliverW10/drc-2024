

// TODO: make T only SimpleDriveCommand or DrivePath
pub trait IDrive<T> {
    fn drive(command: T);
}