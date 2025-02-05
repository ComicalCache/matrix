use core::time::Duration;
use std::thread::sleep;

mod printer;
use printer::Printer;

mod number_streak;
mod xoshiro256p;

type IOResult = Result<(), std::io::Error>;

static mut RUN: bool = true;

fn main() -> IOResult {
    ctrlc::set_handler(move || unsafe { RUN = false }).expect("Failed to install signal handler");

    let mut printer = Printer::new()?;

    // main loop
    while unsafe { RUN } {
        printer.tick()?;

        sleep(Duration::from_millis(25));
    }

    Ok(())
}
