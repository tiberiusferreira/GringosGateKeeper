extern crate sysfs_gpio;
extern crate chrono;

mod gate_open_noise_filter;
pub use self::gate_open_noise_filter::*;
use log::info;
use self::chrono::prelude::*;
use self::sysfs_gpio::{Pin, Direction};
use std::thread::sleep;
use std::time::Duration;
use std::process::{Command};
use failure::{Error};

const GATE_OPEN_SENSOR: u64 = 16;
const GATE: u64 = 20;
const SPOTLIGHT: u64 = 21;

pub struct Hardware {
    gate: Pin,
    spotlight: Pin,
    pub gate_open_sensor: Pin,
    spotlight_on_count: i64,
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
        sleep(Duration::from_millis(500));
        gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0 on startup", GATE));

        let gate_open_sensor = Pin::new(GATE_OPEN_SENSOR);
        gate_open_sensor.export().expect(&format!("Could not export pin {} to user space.", GATE_OPEN_SENSOR));
        sleep(Duration::from_millis(500));
        gate_open_sensor.set_direction(Direction::In).expect(&format!("Could not set pin {} direction to Out", GATE_OPEN_SENSOR));

        let spotlight = Pin::new(SPOTLIGHT);
        spotlight.export().expect(&format!("Could not export pin {} to user space.", SPOTLIGHT));
        sleep(Duration::from_millis(500));
        spotlight.set_direction(Direction::Out).expect(&format!("Could not set pin {} direction to Out", SPOTLIGHT));
        spotlight.set_value(0).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0 on startup", GATE));


        Hardware {
            gate,
            spotlight,
            gate_open_sensor,
            spotlight_on_count: 0,
        }
    }

    pub fn take_pic(&mut self) -> Result<String, Error>{
//        let file_name = "rep_now.jpg";
//        let dt = chrono::Local::now();
//        if !self.gate_is_open() {
//            if dt.hour() <= 7 || dt.hour() >= 17 {
//                self.turn_on_spotlight();
//            }
//        }
//        let status = Command::new("sh")
//            .arg("-c")
//            .arg(format!("fswebcam -S 5 -r 640x480 --flip v --flip h {}", file_name))
//            .status();
//        if !self.gate_is_open() {
//            self.turn_off_spotlight();
//        }
//        let status = status?;
//        if status.success(){
//            return Ok(file_name.to_string());
//        }else{
//            return Err(CameraCaptureError::CameraCaptureError{
//                code: status.code()
//            })?;
//        }
        Ok("aa".to_string())
    }

    pub fn start_listening_gate_state_change(&mut self, call_on_state_change: Box<dyn Fn(NewGateState) -> () + Send>){
        let gate_open_noise_filter = GateOpenNoiseFilter::new(
            self.gate_open_sensor.clone(),
                                                         call_on_state_change)
            .start_getting_gate_state();
    }

    pub fn gate_is_open(&self) -> bool{
        return self.gate_open_sensor.get_value().expect(&format!("Could not get value of GATE_OPEN_SENSOR PIN {}", GATE_OPEN_SENSOR)) == 0;
    }

    pub fn unlock_gate(&self){
        self.gate.set_value(1).expect(&format!("Could not set GATE_PIN_NUMBER {} to 1", GATE));
        sleep(Duration::from_millis(500));
        self.gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0", GATE));
    }

    pub fn allow_lock(&self){
        self.gate.set_value(0).expect(&format!("Could not set GATE_PIN_NUMBER {} to 0", GATE));
    }

    pub fn turn_on_spotlight(&mut self){
        self.spotlight_on_count += 1;
        info!("spotlight_on_count increased to {}", self.spotlight_on_count);
        self.spotlight.set_value(1).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
    }

    pub fn turn_off_spotlight(&mut self){
        self.spotlight_on_count -= 1;
        info!("spotlight_on_count decreased to {}", self.spotlight_on_count);
        if self.spotlight_on_count < 0{
            error!("spotlight_on_count negative! {}", self.spotlight_on_count);
        }
        if self.spotlight_on_count == 0 {
            self.spotlight.set_value(0).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
        }
    }

    pub fn emergency_turn_off_spotlight(&self){
        self.spotlight.set_value(0).expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
    }


}
