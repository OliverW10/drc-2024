#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include <functional>

namespace Encoder{


const uint PIN_A = 1;
const uint PIN_B = PIN_A+1;   

const uint32_t PINS_MASK = 1 << PIN_A & 1 << PIN_B;

int getIndex(char state){
    // 00 01 11 10
    int order_mapping[4] = {0, 1, 3, 2};
    return order_mapping[state & 0b11];
}

int getDiff(char oldState, char new_state) {
    int diff = getIndex(new_state) - getIndex(oldState);
    // handle wrapping backwards
    if(diff > 2){
        diff -= 4;
    }
    // handle wrapping forwards
    if(diff <= -2){
        diff += 4;
    }
    return diff;
}

// Would make this stuff a class but the irq callback becomes annoying if its a method

int steps = 0;
int64_t step_time_us = 0;
absolute_time_t last_time;

char last_state = 0;
double step_time_alpha = 0.4;

void init(){
    gpio_set_dir(PIN_A, false);
    gpio_set_dir(PIN_B, false);
    gpio_set_irq_enabled_with_callback(PIN_A, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true, &handler);
    gpio_set_irq_enabled_with_callback(PIN_B, GPIO_IRQ_EDGE_RISE | GPIO_IRQ_EDGE_FALL, true, &handler);
}

const int64_t debounce_time_diff_us = 5000;

void handler(uint gpio, uint32_t event_mask){
    absolute_time_t current_time = get_absolute_time();

    char current_state = (gpio_get_all() & PINS_MASK) >> PIN_A;
    // any bouncing should cancel out by using the state diff's so don't need to debounce this
    steps += getDiff(last_state, current_state);

    int64_t time_diff = absolute_time_diff_us(last_time, current_time);
    if(time_diff < debounce_time_diff_us){
        return;
    }

    if(step_time_us != 0){
        step_time_us = step_time_us * (1 - step_time_alpha) + time_diff * step_time_alpha;
    }else {
        // first loop
        step_time_us = time_diff;
    }
    last_time = current_time;
}

const double WHEEL_DIAM = 0.04;
const double WHEEL_CIRCUM = WHEEL_DIAM * 3.1415;
const double METERS_PER_STEP = WHEEL_CIRCUM / 4; // 4 steps per revolution in a quadrature encoder

// Meters
double getDistance(){
    return steps * METERS_PER_STEP;
}

const double SECONDS_IN_US = 0.001 * 0.001;

// Meters per second
double getSpeed(){
    return METERS_PER_STEP / (step_time_us * SECONDS_IN_US); 
}


/*
00 00 -> 0
00 01 -> 1
00 10 -> -1
00 11 -> 2?

01 00 -> -1
01 01 -> 0
01 10 -> 2?
01 11 -> 1

10 00 -> 1
10 01 -> 2?
10 10 -> 0
10 11 -> -1

11 00 -> 2?
11 01 -> -1
11 10 -> 1
11 11 -> 0

*/

// pico example for quadrature
// https://github.com/raspberrypi/pico-examples/tree/master/pio/quadrature_encoder
// uses pio (seperate hardware state machine) aimed at reading high rate (millions / sec) quadrature signals
// mine will likely be <40 / sec so i will just use interupts and timers

}