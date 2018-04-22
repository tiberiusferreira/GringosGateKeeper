extern crate serial;
extern crate flexi_logger;
extern crate teleborg;
extern crate log_panics;
#[macro_use] extern crate log;
#[macro_use] extern crate diesel_infer_schema;
#[macro_use] extern crate diesel;
extern crate failure;

mod database;
mod gatekeeper;
use flexi_logger::{opt_format, Logger};


fn main() {
    log_panics::init();
    Logger::with_str("info")
        .log_to_file()
        .directory("log_files")
        .format(opt_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let mut gk = gatekeeper::GringosGateKeeperBot::<teleborg::Bot>::new();
    gk.start();

}
