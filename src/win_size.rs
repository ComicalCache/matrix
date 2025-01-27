#[repr(C)]
#[derive(Default)]
pub struct WinSize {
    pub rows: u16,
    pub cols: u16,
    /// Unused
    pub __xpixel: u16,
    /// Unused
    pub __ypixel: u16,
}

impl PartialEq for WinSize {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols
    }
}

impl Copy for WinSize {}

impl Clone for WinSize {
    fn clone(&self) -> Self {
        *self
    }
}
