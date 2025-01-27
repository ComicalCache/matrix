pub extern "C" fn int_handler(_: i32) {
    unsafe { crate::RUN = false };
}

unsafe extern "C" {
    pub fn signal(sig: i32, handler: extern "C" fn(i32));
    pub fn ioctl(fd: i32, request: u64, ...) -> i32;
}

#[link(name = "consts", kind = "static")]
unsafe extern "C" {
    pub static tiocgwinsz: u64;
    pub static o_evtonly: i32;
    pub static o_nonblock: i32;

    pub static sigint: i32;
    pub static sigterm: i32;
}
