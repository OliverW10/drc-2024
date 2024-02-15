# DRC-2024

Software for an entry into the 2024 Droid Racing Challenge hosted by QUT.

`/planner` - Vision and path planning code to run on a raspberry pi - Rust
`/micro` - Control code to run on a raspberry pi pico - C++
`/micro-py` - Control code to run on a raspberry pi pico for testing - (Micro) Python
`/shared` - Message interfaces used in communication between other components - Protobuf
`/sim` - Simulation of the car and track to test with - C#
`/client` - Client to control and monitor the car - Typescript
`/arrow` - Machine learning model to detect the turning arrow - Python
`/analysys` - Scripts to analyse and collect data to inform decisions


requires debian packages:
    - protobuf-compiler
