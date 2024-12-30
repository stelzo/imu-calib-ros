# IMU Calibration Tool for ROS2

A minimal ROS2 package for calibrating IMU intrinsics using Gravity on a flat surface. It is a wrapper around the [imu-calib](https://crates.io/crates/imu-calib) crate.

The nodes are written in Rust, follow ½[this instructions](https://github.com/stelzo/ros-dev-setup/tree/main/rust) for installing the Rust + ROS2 toolchain.

## Setup

Clone the repo into your ROS2 workspace and build it with `colcon`.
```bash
cd ~/ros2_ws/src
git clone https://github.com/stelzo/imu-calib-ros
cd ..
colcon build --packages-select imu-calib-ros --cargo-args --release
```

Source the workspace and you are ready to go.

## Usage

Connect the IMU, run your driver node and start the calibration process.
```bash
ros2 run imu-calib-ros calibrate --ros-args -r imu/data_raw:=<your-imu-topic>
```
Follow the instructions on the terminal. The calibration results are printed at the end and saved to a file.

When finished, the calibration can be applied to the IMU data stream.
```bash
ros2 run imu-calib-ros apply --ros-args -r imu/data_raw:=<your-imu-topic> -r imu/data_calibrated:=<your-calibrated-output-topic> -p calib_file:=<your_calibration_file>
```
The calibration file path is relative to your workspace if you don't provide an absolute path and run it from the workspace root.
Use the absolute path if the node cannot find the file.

### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
