extern crate sysfs_gpio;
extern crate chrono;

use self::chrono::prelude::*;
use self::sysfs_gpio::{Pin, Direction};
const GATE_OPEN_SENSOR: u64 = 16;
const GATE: u64 = 20;
const SPOTLIGHT: u64 = 21;
use std::thread::sleep;
use std::time::Duration;
use std::process::{Command};
use failure::{Error};
pub struct Hardware {
    gate: Pin,
    spotlight: Pin,
    gate_open_sensor: Pin
}

#[derive(Debug, Fail)]
pub enum CameraCaptureError {
    #[fail(display = "Camera command exited with non-zero code {:?}", code)]
    CameraCaptureError{
        code: Option<i32>
    }
}

impl Hardware {
    pub fn new() -> Self{
        let gate = Pin::new(GATE);
        gate.export().expect(&format!("Could not export pin {} to user space.", GATE));
        sleep(Duration::from_millis(500));
        gate.set_direction(Direction::Out).expect(&format!("Could not set pin {} direction to Out", GATE));

        let gate_open_sensor = Pin::new(GATE_OPEN_SENSOR);
        gate_open_sensor.export().expect(&format!("Could not export pin {} to user space.", GATE_OPEN_SENSOR));
        sleep(Duration::from_millis(500));
        gate_open_sensor.set_direction(Direction::In).expect(&format!("Could not set pin {} direction to Out", GATE_OPEN_SENSOR));

        let spotlight = Pin::new(SPOTLIGHT);
        spotlight.export().expect(&format!("Could not export pin {} to user space.", SPOTLIGHT));
        sleep(Duration::from_millis(500));
        spotlight.set_direction(Direction::Out).expect(&format!("Could not set pin {} direction to Out", SPOTLIGHT));
        spotlight.set_value(0).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0 on startup", GATE));
        gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0 on startup", GATE));
        Hardware {
            gate,
            spotlight,
            gate_open_sensor
        }
    }

    pub fn take_pic(&self) -> Result<String, Error>{
        let file_name = "rep_now.jpg";
        let dt = chrono::Local::now() ;
        if dt.hour() < 7 || dt.hour() > 18 {
            self.turn_on_spotlight();
        }
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!("fswebcam -S 5 -r 640x480 --flip v --flip h {}", file_name))
            .status();
        self.turn_off_spotlight();
        let status = status?;
        if status.success(){
            return Ok(file_name.to_string());
        }else{
            return Err(CameraCaptureError::CameraCaptureError{
                code: status.code()
            })?;
        }
    }

    pub fn gate_is_open(&self) -> bool{
        return self.gate_open_sensor.get_value().expect(&format!("Could not get value of GATE_OPEN_SENSOR PIN {}", GATE_OPEN_SENSOR)) == 0;
    }

    pub fn open_gate(&self){
        self.gate.set_value(1).expect(&format!("Could not set GATE_PIN_NUMBER {} to 1", GATE));
        sleep(Duration::from_millis(500));
        self.gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0", GATE));
    }

    pub fn close_gate(&self){
        self.gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0", GATE));
    }

    pub fn turn_on_spotlight(&self){
        self.spotlight.set_value(1).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
    }
    pub fn turn_off_spotlight(&self){
        self.spotlight.set_value(0).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
    }

}
