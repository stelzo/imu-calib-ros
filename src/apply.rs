use std::sync::{Arc, Mutex};

use futures_lite::{future, StreamExt};
use r2r::{sensor_msgs::msg::Imu, QosProfile, RosParams};

#[derive(Debug, Clone, RosParams)]
struct Params {
    calib_file: String,
    calibrate_gyros: bool,
    gyro_calib_samples: i32,
    gravity: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            calib_file: "calib.txt".to_string(),
            calibrate_gyros: true,
            gyro_calib_samples: 100,
            gravity: 9.80665,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "imu_calib_ros_apply", "")?;

    let config = Arc::new(Mutex::new(Params::default()));
    let (_, _) = node.make_derived_parameter_handler(config.clone())?;

    let (calibrate_gyros, gyro_calib_samples, output_file, gravity) = {
        let config = config.lock().unwrap();

        (
            config.calibrate_gyros,
            config.gyro_calib_samples,
            config.calib_file.clone(),
            config.gravity,
        )
    };

    let mut calib = imu_calib::CalibrationApplication::create(
        calibrate_gyros,
        gyro_calib_samples as usize,
        output_file,
        gravity,
    )?;

    let imu_pub = node.create_publisher::<Imu>("imu/data_calibrated", QosProfile::default())?;
    let imu_sub = node.subscribe::<Imu>("imu/data_raw", QosProfile::default())?;
    std::thread::spawn(move || {
        future::block_on(async {
            imu_sub
                .for_each(|msg| {
                    let mut calib_msg = imu_calib::ImuMsg::default();
                    calib_msg.angular_velocity[0] = msg.angular_velocity.x;
                    calib_msg.angular_velocity[1] = msg.angular_velocity.y;
                    calib_msg.angular_velocity[2] = msg.angular_velocity.z;
                    calib_msg.linear_acceleration[0] = msg.linear_acceleration.x;
                    calib_msg.linear_acceleration[1] = msg.linear_acceleration.y;
                    calib_msg.linear_acceleration[2] = msg.linear_acceleration.z;
                    let corrected = calib.imu_callback(calib_msg);
                    if let Some(corrected) = corrected {
                        let mut imu_msg = Imu::default();
                        imu_msg.angular_velocity.x = corrected.angular_velocity[0];
                        imu_msg.angular_velocity.y = corrected.angular_velocity[1];
                        imu_msg.angular_velocity.z = corrected.angular_velocity[2];
                        imu_msg.linear_acceleration.x = corrected.linear_acceleration[0];
                        imu_msg.linear_acceleration.y = corrected.linear_acceleration[1];
                        imu_msg.linear_acceleration.z = corrected.linear_acceleration[2];
                        match imu_pub.publish(&imu_msg) {
                            Ok(_) => {}
                            Err(e) => {
                                r2r::log_error!("imu_calib_ros_apply", "{}", e);
                            }
                        }
                    }
                })
                .await;
        });
    });

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
    }
}
