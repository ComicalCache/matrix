use core::time::Duration;
use std::thread::sleep;

use ctrlc;

mod printer;
use printer::Printer;

mod number_streak;
mod xoshiro256p;

type IOResult = Result<(), std::io::Error>;

static mut RUN: bool = true;

fn main() -> IOResult {
    ctrlc::set_handler(move || unsafe { RUN = false }).expect("Failed to install signal handler");

    let mut printer = Printer::new();
    printer.init()?;

    // main loop
    while unsafe { RUN } {
        printer.tick()?;

        sleep(Duration::from_millis(25));
    }

    printer.deinit()
}
