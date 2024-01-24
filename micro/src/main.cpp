#include "pico/stdlib.h"
#include <stdio.h>

// void main(){
//     // read serial comms

//     // read imu
//     // read wheel encoder
//     // set drive

//     // read servo pwm
//     // do servo pid

//     // send odom comms
// }

int main() {
    const uint LED_PIN = PICO_DEFAULT_LED_PIN;
    stdio_init_all();
    gpio_init(LED_PIN);
    gpio_set_dir(LED_PIN, GPIO_OUT);
    while (true) {
        printf("Hello world from serial\n");
        gpio_put(LED_PIN, 1);
        sleep_ms(500);
        gpio_put(LED_PIN, 0);
        sleep_ms(500);
    }
}
