use core::time::Duration;
use std::os::fd::AsRawFd;
use std::thread::sleep;

use std::os::unix::fs::OpenOptionsExt;

mod printer;
use printer::Printer;

mod rgb_color;
use rgb_color::RGBColor;

mod number_streak;
mod win_size;
mod xoshiro256p;

// ffi c bindings
mod c;

type IOResult = Result<(), std::io::Error>;

static mut RUN: bool = true;

fn main() -> IOResult {
    // install handler for ctrl+c
    unsafe { c::signal(c::sigint, c::int_handler) };
    unsafe { c::signal(c::sigterm, c::int_handler) };

    // open tty
    let tty = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(unsafe { c::o_evtonly | c::o_nonblock })
        // macOS uses /dev/tty instead of STDOUT
        .open("/dev/tty")
        .expect("Failed to open tty");

    let mut printer = Printer::new(tty.as_raw_fd(), RGBColor(20, 20, 20));
    printer.init()?;

    // main loop
    while unsafe { RUN } {
        printer.tick()?;

        sleep(Duration::from_millis(25));
    }

    printer.deinit()
}
