extern crate sysfs_gpio;
use self::sysfs_gpio::{Pin, Direction, PinPoller, Edge};
use std::{thread, time::Duration};

pub struct GateOpenNoiseFilter{
    pub gate_open_pin: Pin,
    pub gate_open_pin_poller: PinPoller,
    call_on_state_change: Box<dyn Fn(NewGateState) -> () + Send>
}

#[derive(Debug, PartialEq, Clone)]
pub enum NewGateState {
    OPEN,
    CLOSED
}

impl GateOpenNoiseFilter{
    pub fn new(gate_open_sensor: Pin, call_on_state_change: Box<dyn Fn(NewGateState) -> () + Send>) -> Self{
        let poller = gate_open_sensor.get_poller().expect("Could not get gate_open poller, maybe pin does not support interrupts");
        GateOpenNoiseFilter{
            gate_open_pin: gate_open_sensor.clone(),
            gate_open_pin_poller: poller,
            call_on_state_change
        }
    }

    pub fn start_getting_gate_state(mut self){
        thread::spawn(move || {
            self.gate_open_pin.set_edge(Edge::BothEdges)
                .expect("Could not enable both edges interrupt on Gate Open GPIO Pin");
            thread::sleep(Duration::from_millis(500));
            info!("Waiting for an interrupt on gate open pin");
            let mut last_stable_state = self.get_current_state();
            loop {
                info!("Initial gate open GPIO state: {:#?}", last_stable_state);
                last_stable_state = self.block_on_next_stable_state_change(last_stable_state.clone());
                (self.call_on_state_change)(last_stable_state.clone());
            }
        });
    }


    fn map_u8_to_state(pin_value: u8) -> NewGateState {
        if pin_value == 1{
            NewGateState::OPEN
        }else{
            NewGateState::CLOSED
        }
    }

    fn poll_new_state_up_to(&mut self, max_wait_ms: isize) -> Option<NewGateState>{
        let pin_value = self.gate_open_pin_poller.poll(max_wait_ms)
            .unwrap_or_else(|e| panic!("Got error {} while reading GPIO pin.", e))
            .and_then(|new_pin_value| Some(Self::map_u8_to_state(new_pin_value)));
        return pin_value;
    }
    fn get_gpio_change_blocking(&mut self) -> NewGateState {
        info!("Blocking waiting for Pin Poller interrupt");
        let pin_value = self.gate_open_pin_poller.poll(-1)
            .unwrap_or_else(|e| panic!("Got error {} while polling GPIO pin.", e))
            .expect("Got none while polling GPIO, this should never happen since it has -1 as timeout");
        info!("Got interrupt!");
        return Self::map_u8_to_state(pin_value);
    }

    fn get_current_state(&self) -> NewGateState {
        let last_pin_value = self.gate_open_pin.get_value()
            .unwrap_or_else(|e| panic!("Got error {} while reading GPIO pin.", e));
        let last_state = Self::map_u8_to_state(last_pin_value);
        return last_state;
    }

    fn block_on_next_stable_state_change(&mut self, last_stable_state: NewGateState) -> NewGateState {
        'wait_forever_for_gpio_change: loop {
            // Wait until a change occurs
            let mut brand_new_state = self.get_gpio_change_blocking();
            // loop until value is stable
            'wait_for_gpio_stabilization: loop {
                info!("GPIO value changed to {:#?}. Waiting for stabilization.", brand_new_state);
                // Wait up to 500ms for a second change
                let maybe_another_state_change= self.poll_new_state_up_to(500);
                match maybe_another_state_change {
                    Some(another_state_change) => {
                        info!("There was second change in the GPIO value in less than 500ms. Its not stable. New value is {:#?}", another_state_change);
                        brand_new_state = another_state_change;
                        continue 'wait_for_gpio_stabilization;
                    },
                    None => {
                        info!("GPIO did not change for 500ms, declaring it stable");
                        let new_state = self.get_current_state();
                        if new_state != brand_new_state{
                            error!("Value from stable change is different from actual value: Change = {:#?} Actual = {:#?}", brand_new_state, new_state);
                        }
                        if new_state == last_stable_state{
                            info!("New stable value is the same as last one. No actual change occured.");
                            continue 'wait_forever_for_gpio_change;
                        }else {
                            return new_state;
                        }
                    }
                }
            }
        }
    }

}