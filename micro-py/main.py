# Prototype script to test hardware with before writing final version in c++

from machine import Pin, PWM, ADC
import utime
import micropython
import umachine
import network
from time import sleep
import machine
import asyncio
from dataclasses import dataclass
from pure_protobuf.annotations import Field
from pure_protobuf.message import BaseMessage
from typing_extensions import Annotated



############## Steer

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


############## Drive


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



############## Encoder


encoder_pin_a = Pin(14, mode=Pin.IN, pull=Pin.PULL_UP)
encoder_pin_b = Pin(1, mode=Pin.IN, pull=Pin.PULL_UP)

micropython.alloc_emergency_exception_buf(100)

# 00 01 11 10
# 0  1  2  3
state_map = [0, 1, 3, 2]

encoder_steps = 0
last_time = utime.ticks_us()
time_diff = 0
cur_time = 0

def encoder_interupt_handler(pin):
    # TODO: calculate steps diff to be resiliant against bouncing
    global encoder_steps, time_diff, last_time, cur_time

    state = umachine.disable_irq()

    cur_time = utime.ticks_us() # TODO: if this call creates a new python object it will not work
    time_diff = utime.ticks_diff(cur_time, last_time)
    if time_diff > 1000: # 1ms
        encoder_steps += 1
        last_time = cur_time

    umachine.enable_irq(state)


encoder_pin_a.irq(encoder_interupt_handler, trigger=Pin.IRQ_FALLING)
encoder_pin_a.irq(encoder_interupt_handler, trigger=Pin.IRQ_RISING)
encoder_pin_b.irq(encoder_interupt_handler, trigger=Pin.IRQ_FALLING)
encoder_pin_b.irq(encoder_interupt_handler, trigger=Pin.IRQ_RISING)

WHEEL_DIAM = 0.05
ENCODER_STEP_LENGTH = WHEEL_DIAM * 3.1415 / 4
US_TO_S = 1000 * 1000

def current_speed():
    return ENCODER_STEP_LENGTH / (time_diff * US_TO_S)


############## Comms

# TODO: either write .proto->pure_protobuf dataclass code gen or compile google protobuf to pico 
# https://github.com/eigenein/protobuf
@dataclass
class SimpleDrive(BaseMessage):
    speed: Annotated[float, Field(2)] = 0
    curvature: Annotated[float, Field(3)] = 0


from wifi_creds import WIFI_PASS, WIFI_SSID

def connect():
    #Connect to WLAN
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    wlan.connect(WIFI_SSID, WIFI_PASS)
    while wlan.isconnected() == False:
        print('Waiting for connection...')
        # TODO: toggle led
        sleep(1) # 0.2
    print(wlan.ifconfig())
    return wlan.ifconfig()[0]

last_command = SimpleDrive()
last_command_time_ms = 0

async def handle_client(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    global last_command, last_command_time_ms
    request = None
    while request != 'restart':
        last_command = SimpleDrive.loads(await reader.read(255))
        last_command_time_ms = utime.ticks_ms()
        writer.write(str(utime.ticks_ms()).encode())
        await writer.drain()
    writer.close()

async def run_server():
    server = await asyncio.start_server(handle_client, 'localhost', 8001)
    async with server:
        await server.serve_forever()

try:
    ip = connect()
except KeyboardInterrupt:
    machine.reset()



async def main():
    while True:
        time_since_last_command_ms = utime.ticks_diff(utime.ticks_ms(), last_command_time_ms)
        if time_since_last_command_ms > 100:
            print("Command Timeout")
            await asyncio.sleep(0.2)
            continue

        ## Steer
        # JST SH 5 pin
        target_reading = curvature_to_reading(last_command.curvature)
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
        target_speed = last_command.speed
        drive_effort = calc_drive_ff(target_speed) + calc_drive_p(target_speed, current_speed())
        drive_pwm.duty_ns(effort_to_ns(drive_effort))

        ## IMU
        # read values from imu over i2c
        # use kalman filter to combine encoder and imu readings

        # asyncio.sleep_ms(10)
        await asyncio.sleep(0.02)

asyncio.gather(main(), run_server())

# TODO:
# - imu