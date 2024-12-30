use std::sync::{Arc, Mutex};

use futures_lite::{future, StreamExt};
use r2r::{sensor_msgs::msg::Imu, QosProfile, RosParams};

#[derive(Debug, Clone, RosParams)]
struct Params {
    calib_file: String,
    gravity: f64,
    measurements_per_orientation: i32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            calib_file: "calib.txt".to_string(),
            gravity: 9.80665,
            measurements_per_orientation: 500,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "imu_calib_ros_calibrate", "")?;

    let config = Arc::new(Mutex::new(Params::default()));
    let (_, _) = node.make_derived_parameter_handler(config.clone())?;

    let (measurements_per_orientation, reference_acceleration, output_file) = {
        let config = config.lock().unwrap();

        (
            config.measurements_per_orientation,
            config.gravity,
            config.calib_file.clone(),
        )
    };

    let calib = Arc::new(Mutex::new(imu_calib::CalibrateProcess::new(
        measurements_per_orientation as usize,
        reference_acceleration,
        output_file,
    )));

    let imu_sub = node.subscribe::<Imu>("imu/data_raw", QosProfile::default())?;
    let calib_inner = calib.clone();
    std::thread::spawn(move || {
        future::block_on(async {
            imu_sub
                .for_each(|msg| {
                    let mut calib = calib_inner.lock().unwrap();
                    let mut calib_msg = imu_calib::ImuMsg::default();
                    calib_msg.angular_velocity[0] = msg.angular_velocity.x;
                    calib_msg.angular_velocity[1] = msg.angular_velocity.y;
                    calib_msg.angular_velocity[2] = msg.angular_velocity.z;
                    calib_msg.linear_acceleration[0] = msg.linear_acceleration.x;
                    calib_msg.linear_acceleration[1] = msg.linear_acceleration.y;
                    calib_msg.linear_acceleration[2] = msg.linear_acceleration.z;
                    match calib.imu_callback(calib_msg) {
                        Ok(_) => {}
                        Err(e) => {
                            r2r::log_error!("imu_calib_ros_calibrate", "{}", e);
                        }
                    }
                })
                .await;
        });
    });

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        let calib = calib.lock().unwrap();
        if calib.is_done() {
            break;
        }
    }

    Ok(())
}
