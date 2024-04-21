# DRC-2024

Software for one of UTS's enties into the 2024 Droid Racing Challenge hosted by QUT.

- `/planner` - Vision and path planning code to run on a raspberry pi - Rust
- `/shared` - Message interfaces used in communication between other components - Protobuf
- `/arrow` - Machine learning model to detect the turning arrow - Python
- `/micro-py` - Control code to run on a raspberry pi pico for testing - (Micro) Python
- `/micro` - Control code to run on a raspberry pi pico - C++

requires debian packages:
    - libclang-dev
    - libopencv-dev
    - protobuf-compiler
