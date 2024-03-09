need a pico (or other microcontroller) to use an adc to measure battery voltage
need a full pi for csi camera port

# Sections
## Controller: C++ - microcontroller code to run on pi pico
- Encoder
    - In front diff housing - may want to put front diff back to spread load if back diff is skipping too much
    - On wheel(s) - Would either need two or have to account for cornering in software, or use the ostrich algorithm
    - Optical quadrature - interupts
- IMU
    - read over i2c
    - Do sensor fusion of the imu and encoder to get a pose
- Follow path OR simple control
    - gets path as a series of positions with speeds
    - use pure pursuit to follow
- Control
    - Read servo encoder and do pid
    - do pid for speed with main esc
- Send pose to planner

## Planner: Rust? - vision code to run on pi zero, generates path



## Client: python - gui for control and visualize data, should look at graphical gui options in c#
## Sim: Unity C# - Track generator and physics sim to test with, if physical car is ready soon may not be nessisary, would be needed if I want to explore machine learning approaches in the future
Shared: protobuf messages to go between Planner, Controller, Client and sim

Controller <-> Planner <-> Client
sim replaces controller and client, using protobuf makes this easy and simplifies the language barrier
ros?


https://core-electronics.com.au/raspberry-pi-wide-angle-camera-module-seeed-studio.html
https://www.uctronics.com/index.php/ov5647-noir-camera-board-w-m12x0-5-mount-for-raspberry-pi.html

Should I have be flashing the pico from my laptop or from the pi? if from the pi, should check that you are able to use the same usb connection for serial comms and then for flashing?S

vision -> plan path -> path follower -> drive controller
[       pi zero      ][            pi pico             ]
vision: perspective corrected point cloud
plan path: a* in phase space
path follower: pure pursit
drive controller: pid's and tire slip lookup

https://www.altronics.com.au/p/z6473-raspberry-pi-5mp-fisheye-camera-module/ - seems that the raspberry pi zero has a different camera connector to what the normal raspberry pi has, this camer ahas the smalller one that the raspberry pi zero uses
https://core-electronics.com.au/raspberry-pi-wide-angle-camera-module-seeed-studio.html - if using with the pi zero may have to get a cable adapter, otherwise can just use a pi 3

https://github.com/rust-cv/cv
https://medium.com/@kennethjiang/calibrate-fisheye-lens-using-opencv-333b05afa0b0
