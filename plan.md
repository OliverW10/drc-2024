need a pico (or other microcontroller) to use an adc to measure battery voltage
need a full pi for csi camera port

# Sections
## Controller: C++ - microcontroller code to run on pi pico, does odmoetry by motor mesuring voltage and using imu and runs pathfollowing
- Encoder
    - ~~From motor with back-emf~~ - difficult because would have to time adc measurments with pwm signal troughs, would have to somehow work out the phase the esc is generating
    - In front diff housing - may want to put front diff back to spread load if back diff is skipping too much
    - On wheel(s) - Would either need two or have to account for cornering in software

## Planner: Rust? - vision code to run on pi zero, generates path


## Client: python - gui for control and visualize data, should look at graphical gui options in c#
## Sim: Unity C# - Track generator and physics sim to test with, if physical car is ready soon may not be nessisary, would be needed if I want to explore machine learning approaches in the future
Shared: protobuf messages to go between Planner, Controller, Client and sim

Controller <-> Planner <-> Client
sim replaces controller and client, using protobuf makes this easy and simplifies the language barrier
ros?
