#include "steer.h"
#include "hardware/adc.h"

namespace Steer{

void init(){
    // https://github.com/raspberrypi/pico-examples/blob/master/adc/hello_adc/hello_adc.c
    adc_init();
    adc_gpio_init(26);
    adc_select_input(0);
}

const double MAX_CURVATURE = 0.8;

const uint16_t MIN_TURN_READING = 2000;
const uint16_t MAX_TURN_READING = 4000;

const uint16_t TURN_READING_HALF_RANGE = (MAX_TURN_READING - MIN_TURN_READING) / 2;
const uint16_t TURN_MID = (MAX_TURN_READING / 2 + MIN_TURN_READING / 2);

uint16_t curvatureToPosition(double curvature){
    return TURN_MID + (curvature / MAX_CURVATURE) * TURN_READING_HALF_RANGE;
}

void steer(double target_curvature){
    uint16_t position = adc_read();
    uint16_t target_position = curvatureToPosition(target_curvature);
}

}