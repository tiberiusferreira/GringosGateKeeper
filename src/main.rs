extern crate serial;
extern crate flexi_logger;
extern crate teleborg;
extern crate log_panics;
#[macro_use] extern crate log;
#[macro_use] extern crate diesel_infer_schema;
#[macro_use] extern crate diesel;
#[macro_use] extern crate failure;

mod database;
mod gatekeeper;
mod hardware;
use flexi_logger::{opt_format, Logger};

// Holofone : 21
// Portao : 20

//echo "1" > /sys/class/gpio/gpio21/value

fn main() {
    log_panics::init();
    Logger::with_str("info")
        .log_to_file()
        .directory("log_files")
        .format(opt_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let old_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        old_hook(panic);
        gatekeeper::GringosGateKeeperBot::<teleborg::Bot>::emergency_turn_off();
    }));

    let mut gk = gatekeeper::GringosGateKeeperBot::<teleborg::Bot>::new();
    gk.start();
}


