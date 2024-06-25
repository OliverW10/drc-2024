# DRC-2024

Software for one of UTS's enties into the 2024 Droid Racing Challenge hosted by QUT.

- `/planner` - Vision and path planning code to run on a raspberry pi - Rust
- `/shared` - Message interfaces used in communication between other components - Protobuf
- `/arrow` - Machine learning model to detect the turning arrow - Python
- `/micro-py` - Control code to run on a raspberry pi pico for testing - (Micro) Python
- `/micro` - Control code to run on a raspberry pi pico - C++

requires debian packages: libclang-dev clang libopencv-dev protobuf-compiler
fixed camera not working (but showing up to libcamera-hello --list-cameras) by doing rpi-update and using camera_auto_detect=1 in /boot/firmware/config.txt
fix gpio with https://docs.rs/rppal/latest/rppal/pwm/index.html
