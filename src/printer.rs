use std::io::{Stdout, Write, stdout};

use crate::{
    IOResult, c,
    number_streak::{NumberStreak, Pos, STREAK_LENGTH},
    rgb_color::RGBColor,
    win_size::WinSize,
    xoshiro256p::Xoshiro256pState,
};

const COLORS: [RGBColor; STREAK_LENGTH] = [
    RGBColor(5, 225, 19),
    RGBColor(34, 213, 25),
    RGBColor(46, 201, 30),
    RGBColor(54, 189, 34),
    RGBColor(59, 178, 37),
    RGBColor(63, 166, 39),
    RGBColor(65, 155, 41),
    RGBColor(66, 144, 43),
    RGBColor(66, 133, 44),
    RGBColor(66, 122, 45),
    RGBColor(65, 112, 46),
    RGBColor(63, 101, 46),
    RGBColor(61, 91, 46),
    RGBColor(59, 81, 46),
    RGBColor(56, 71, 46),
    RGBColor(52, 61, 46),
];

pub struct Printer {
    /// Background color
    background: RGBColor,

    /// Terminal size
    size: WinSize,
    size_changed: bool,

    /// Streaks to be drawn
    streaks: Vec<NumberStreak>,

    pending_streaks: Vec<usize>,
    initialized_streaks: Vec<usize>,
    dead_streaks: Vec<usize>,

    /// TTY file descriptor (needed for ioctl)
    tty_fd: i32,

    /// stdout handle
    stdout: Stdout,

    /// Random number generator
    xoshiro256p: Xoshiro256pState,
}

/// Public methods
impl Printer {
    pub fn new(tty_fd: i32, background: RGBColor) -> Self {
        Self {
            background,

            size: WinSize::default(),
            size_changed: false,

            streaks: Vec::new(),
            pending_streaks: Vec::new(),
            initialized_streaks: Vec::new(),
            dead_streaks: Vec::new(),

            tty_fd,

            stdout: stdout(),

            xoshiro256p: Xoshiro256pState::new(0xdeadbeef),
        }
    }

    pub fn init(&mut self) -> IOResult {
        self.fetch_size()?;
        self.reinit()?;
        self.hide_cursor()
    }

    pub fn deinit(&mut self) -> IOResult {
        self.show_cursor()?;
        self.reset_fg_color()?;
        self.reset_bg_color()
    }

    pub fn tick(&mut self) -> IOResult {
        self.fetch_size()?;

        // re-init everything after size change
        if self.size_changed {
            self.reinit()?;
        }

        // can't iter over initialized_streaks direcly because of ownership with clear_pos
        for idx in 0..self.initialized_streaks.len() {
            let streak_idx = self.initialized_streaks[idx];

            // clear prevous tail piece
            let row = self.wrapping_row_div(
                self.streaks[streak_idx].row(),
                self.streaks[streak_idx].len(),
            );
            self.clear_pos(row, self.streaks[streak_idx].col())?;

            // draw streak, this must never break the loop!
            for (char_idx, c) in self.streaks[streak_idx].enumerate() {
                let row = self.wrapping_row_div(self.streaks[streak_idx].row(), char_idx as u16);
                self.set_pos(row, self.streaks[streak_idx].col(), c, COLORS[char_idx])?;
            }

            // advance streak (wrapping)
            self.streaks[streak_idx].extend(self.size.rows);

            if self.streaks[streak_idx].is_dead() {
                self.dead_streaks.push(idx);
            }
        }

        // remove dead streaks from intialized and add them to pending
        while let Some(idx) = self.dead_streaks.pop() {
            self.pending_streaks
                .push(self.initialized_streaks.remove(idx));
        }

        // initialize streaks after re-initing vector or removing streak
        if let Some(idx) = self.pending_streaks.pop() {
            let pos = Pos {
                // random row
                row: self.xoshiro256p.next() as u16 % self.size.rows,
                // col determined by shuffled idxs vector + transmute for skipped cols
                col: (idx as u16 * 2) + 1,
            };

            self.streaks[idx].init(pos, self.xoshiro256p.next());
            self.initialized_streaks.push(idx);
        }

        self.stdout.flush()?;

        Ok(())
    }
}

/// Private methods
impl Printer {
    fn fetch_size(&mut self) -> IOResult {
        let old_size = self.size;

        match unsafe { c::ioctl(self.tty_fd, c::tiocgwinsz, &mut self.size) } {
            0 => {}
            err => return Err(std::io::Error::other(format!("ioctl: {err}"))),
        }

        self.size_changed = self.size != old_size;

        Ok(())
    }

    fn reinit(&mut self) -> IOResult {
        self.set_background()?;

        // only fill every other column and skip the first and last one
        self.streaks
            .resize_with((self.size.cols as usize - 2) / 2, Default::default);

        self.pending_streaks = (0..(self.size.cols as usize - 2) / 2).collect();
        self.initialized_streaks.clear();

        // "shuffle" number idxs by permutating elements
        for _ in 0..self.pending_streaks.len() {
            let idx = self.xoshiro256p.next() as usize % self.pending_streaks.len();
            let jdx = self.xoshiro256p.next() as usize % self.pending_streaks.len();

            self.pending_streaks.swap(idx, jdx);
        }

        Ok(())
    }

    fn clear_pos(&mut self, row: u16, col: u16) -> IOResult {
        write!(&mut self.stdout, "{} ", Printer::set_cursor(row, col),)
    }

    fn set_pos(&mut self, row: u16, col: u16, c: char, rgb: RGBColor) -> IOResult {
        let pos = Printer::set_cursor(row, col);
        write!(&mut self.stdout, "{pos}{}{c}", Printer::set_fg_color(rgb))
    }

    fn set_background(&mut self) -> IOResult {
        let pos = Printer::set_cursor(0, 0);
        let fill = " ".repeat(self.size.cols as usize * self.size.rows as usize);
        let background = Printer::set_bg_color(self.background);

        write!(&mut self.stdout, "{pos}{background}{fill}")
    }

    fn show_cursor(&mut self) -> IOResult {
        write!(&mut self.stdout, "\x1b[?25h")
    }

    fn hide_cursor(&mut self) -> IOResult {
        write!(&mut self.stdout, "\x1b[?25l")
    }

    fn reset_fg_color(&mut self) -> IOResult {
        write!(&mut self.stdout, "\x1b[39m")
    }

    fn reset_bg_color(&mut self) -> IOResult {
        write!(&mut self.stdout, "\x1b[49m")
    }

    fn wrapping_row_div(&self, lhs: u16, rhs: u16) -> u16 {
        if lhs >= rhs {
            lhs - rhs
        } else {
            self.size.rows + lhs - rhs
        }
    }
}

/// Private static functions
impl Printer {
    /// Coordinates start at 0
    fn set_cursor(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row + 1, col + 1)
    }

    fn set_bg_color(color: RGBColor) -> String {
        format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2)
    }

    fn set_fg_color(color: RGBColor) -> String {
        format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2)
    }
}
