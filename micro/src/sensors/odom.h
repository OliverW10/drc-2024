

// Kalman filter
// - Expected behavior based on given inputs (steer angle and drive power)
// - Values from IMU
// - Value from encoder

struct DriveCommand{

}

struct IMU6Axis {

}

struct Pose {
    double x;
    double y;
    double theta;
    double d_x;
    double d_theata;
}

void addImu(IMU6Axis readings);

void addEncoder(double speed);

Pose update(DriveCommand inputs);