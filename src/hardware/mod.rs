use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use sysfs_gpio::{Direction, Pin};
use tokio::sync::RwLock;
use tracing::info;

const GATE: u64 = 26;
const SPOTLIGHT: u64 = 17;

pub struct RealHardware {
    gate: Pin,
    spotlight: Pin,
}

pub struct MockHardware {}

impl MockHardware {
    pub fn new() -> MockHardware {
        Self {}
    }
}

pub struct GateHardwareInner<T: RawHardware> {
    hardware: T,
    instant_to_turn_off: Option<std::time::Instant>,
}

pub struct RefCountedGateHardware<T: RawHardware> {
    inner: Arc<RwLock<GateHardwareInner<T>>>,
}

impl<T: RawHardware> RefCountedGateHardware<T> {
    pub fn new_mock() -> RefCountedGateHardware<MockHardware> {
        RefCountedGateHardware {
            inner: Arc::new(RwLock::new(GateHardwareInner {
                hardware: MockHardware::new(),
                instant_to_turn_off: None,
            })),
        }
    }

    pub fn new_real_hardware() -> RefCountedGateHardware<RealHardware> {
        RefCountedGateHardware {
            inner: Arc::new(RwLock::new(GateHardwareInner {
                hardware: RealHardware::new(),
                instant_to_turn_off: None,
            })),
        }
    }

    fn new_ref_counted(&self) -> RefCountedGateHardware<T> {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    pub async fn is_spotlight_on(&self) -> bool {
        self.inner.read().await.instant_to_turn_off.is_some()
    }

    async fn check_spotlight_should_be_turned_off(&self) {
        let mut write_ref = self.inner.write().await;
        if let Some(instant_to_turn_off) = &write_ref.instant_to_turn_off {
            if &std::time::Instant::now() >= instant_to_turn_off {
                info!("Its time to turn off spotlight, turning it off");
                write_ref.hardware.turn_off_spotlight();
                write_ref.instant_to_turn_off.take();
            } else {
                info!("Its not time yet to turn off spotlight");
            }
        } else {
            info!("There is no time to turn off the spotlight, must already be off");
        }
    }

    pub async fn unlock_gate(&self) {
        self.inner.write().await.hardware.unlock_gate();
    }

    pub async fn turn_on_spotlight(&self, duration: Duration) {
        let mut write_lock = self.inner.write().await;
        write_lock.hardware.turn_on_spotlight();
        write_lock.instant_to_turn_off = Some(std::time::Instant::now() + duration);
        let self_ref = self.new_ref_counted();
        tokio::task::spawn(async move {
            tokio::time::sleep(duration).await;
            info!("Checking if its time to turn off spotlight");
            self_ref.check_spotlight_should_be_turned_off().await;
        });
    }
}
impl RawHardware for RealHardware {
    fn unlock_gate(&mut self) {
        self.gate
            .set_value(1)
            .expect(&format!("Could not set GATE_PIN_NUMBER {} to 1", GATE));
        sleep(Duration::from_millis(500));
        self.gate
            .set_value(0)
            .expect(&format!("Could not set GATE_PIN_NUMBER {} to 0", GATE));
    }

    fn turn_on_spotlight(&mut self) {
        self.spotlight
            .set_value(1)
            .expect(&format!("Could not set SPOTLIGHT_PIN {} to 0", SPOTLIGHT));
    }

    fn turn_off_spotlight(&mut self) {
        self.spotlight
            .set_value(0)
            .expect(&format!("Could not set SPOTLIGHT_PIN {} to 1", SPOTLIGHT));
    }
}
impl RawHardware for MockHardware {
    fn unlock_gate(&mut self) {
        sleep(Duration::from_millis(500));
        info!("Unlocked");
    }

    fn turn_on_spotlight(&mut self) {
        sleep(Duration::from_millis(500));
        info!("Spotlight On");
    }

    fn turn_off_spotlight(&mut self) {
        sleep(Duration::from_millis(500));
        info!("Spotlight Off");
    }
}

pub trait RawHardware: Send + Sync + 'static {
    fn unlock_gate(&mut self);

    fn turn_on_spotlight(&mut self);

    fn turn_off_spotlight(&mut self);
}

impl RealHardware {
    pub fn new() -> Self {
        let gate = Pin::new(GATE);
        gate.export()
            .expect(&format!("Could not export pin {} to user space.", GATE));
        sleep(Duration::from_millis(500));
        gate.set_direction(Direction::Out)
            .expect(&format!("Could not set pin {} direction to Out", GATE));
        sleep(Duration::from_millis(500));
        gate.set_value(0).expect(&format!(
            "Could not set GATE_PIN_NUMBER {} to 0 on startup",
            GATE
        ));

        let spotlight = Pin::new(SPOTLIGHT);
        spotlight.export().expect(&format!(
            "Could not export pin {} to user space.",
            SPOTLIGHT
        ));
        sleep(Duration::from_millis(500));
        spotlight
            .set_direction(Direction::Out)
            .expect(&format!("Could not set pin {} direction to Out", SPOTLIGHT));
        spotlight.set_value(0).expect(&format!(
            "Could not set SPOTLIGHT_PIN {} to 0 on startup",
            GATE
        ));

        RealHardware { gate, spotlight }
    }
}
