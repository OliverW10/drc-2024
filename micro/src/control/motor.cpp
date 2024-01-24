#include "pico/stdlib.h"
#include "hardware/pwm.h"

namespace Motor {

uint OUT_PIN = 4;

// https://github.com/raspberrypi/pico-examples/tree/master/pwm/led_fade
// https://github.com/vha3/Hunter-Adams-RP2040-Demos/tree/master/PWM

void init(){
    // Tell the LED pin that the PWM is in charge of its value.
    gpio_set_function(OUT_PIN, GPIO_FUNC_PWM);
    // Figure out which slice we just connected to the LED pin
    uint slice_num = pwm_gpio_to_slice_num(OUT_PIN);
    pwm_config config = pwm_get_default_config();
    config.
}

const double kFF = 1;

void drive(double targetMetersPerSecond, double currentMetersPerSecond){
    // pwm_set_gpio_level(PICO_DEFAULT_LED_PIN, );   
}

}