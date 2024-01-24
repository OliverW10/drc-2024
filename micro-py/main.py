from machine import Pin, PWM, ADC
import utime

steer_analog = ADC(28)
# pot is between white and yello wires


MAX_CURVATURE = 0.8

MIN_TURN_READING = 2000
MAX_TURN_READING = 4000

TURN_READING_HALF_RANGE = (MAX_TURN_READING - MIN_TURN_READING) / 2
TURN_MID = (MAX_TURN_READING / 2 + MIN_TURN_READING / 2)

def curvature_to_reading(curvature):
    return TURN_MID + (curvature / MAX_CURVATURE) * TURN_READING_HALF_RANGE

STEER_kP = 0.05 # guess

steer_pwm_a = PWM(Pin(15))
steer_pwm_a.freq(1000)
steer_pwm_b = PWM(Pin(16))
steer_pwm_b.freq(1000)


drive_pwm = PWM(Pin(17))
drive_pwm.freq(50)

DRIVE_kFF = 0.2

DRIVE_STOP_NS = 1500 * 1000
DRIVE_FULL_NS = 2000 * 1000

DRIVE_RANGE_NS = DRIVE_FULL_NS - DRIVE_STOP_NS

def calc_drive_ff(speed_mps):
    return speed_mps * DRIVE_kFF

def calc_drive_p(speed_mps, target_speed_mps):
    return 0

def effort_to_ns(percent):
    return DRIVE_STOP_NS + percent * DRIVE_RANGE_NS

def main():
    target_curvature = 0
    target_speed = 0
    current_speed = 0
    while True:
        ## Steer
        # JST SH 5 pin
        target_reading = curvature_to_reading(target_curvature)
        steer_reading = steer_analog.read_u16()
        steer_effort = STEER_kP * (steer_reading * target_reading)
        if steer_effort > 0:
            steer_pwm_a.duty_u16(steer_effort)
            steer_pwm_b.duty_u16(0)
        else:
            steer_pwm_a.duty_u16(0)
            steer_pwm_b.duty_u16(-steer_effort)
        print("ADC: ", steer_reading, ", Effort:", steer_effort)

        ## Drive
        drive_effort = calc_drive_ff(target_speed) + calc_drive_p(target_speed, current_speed)
        drive_pwm.duty_ns(effort_to_ns(drive_effort))

        ## IMU
        # read values from imu over i2c
        # use kalman filter to estimate state

        ## Comms
        # revice command (either uart or wifi)
        # send
        
        utime.sleep(0.1)

# TODO:
# - comms
#   - recive
#   - send
# - encoder
# - imu


main()