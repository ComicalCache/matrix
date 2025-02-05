use std::io::{Stdout, Write, stdout};

use termion::{
    color::{self, Rgb},
    cursor::{self, HideCursor},
    screen::{AlternateScreen, IntoAlternateScreen},
    terminal_size,
};

use crate::{
    IOResult,
    number_streak::{NumberStreak, STREAK_LENGTH},
    xoshiro256p::Xoshiro256pState,
};

const COLORS: [Rgb; STREAK_LENGTH] = [
    Rgb(5, 225, 19),
    Rgb(34, 213, 25),
    Rgb(46, 201, 30),
    Rgb(54, 189, 34),
    Rgb(59, 178, 37),
    Rgb(63, 166, 39),
    Rgb(65, 155, 41),
    Rgb(66, 144, 43),
    Rgb(66, 133, 44),
    Rgb(66, 122, 45),
    Rgb(65, 112, 46),
    Rgb(63, 101, 46),
    Rgb(61, 91, 46),
    Rgb(59, 81, 46),
    Rgb(56, 71, 46),
    Rgb(52, 61, 46),
];

pub struct Printer {
    /// Terminal size
    size: (u16, u16),
    size_changed: bool,

    /// Streaks to be drawn
    streaks: Vec<NumberStreak>,

    pending_streaks: Vec<usize>,
    initialized_streaks: Vec<usize>,
    dead_streaks: Vec<usize>,

    /// stdout handle in alternative screen
    stdout: HideCursor<AlternateScreen<Stdout>>,

    /// Random number generator
    xoshiro256p: Xoshiro256pState,
}

/// Public methods
impl Printer {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut printer = Printer {
            size: (0, 0),
            size_changed: false,

            streaks: Vec::new(),
            pending_streaks: Vec::new(),
            initialized_streaks: Vec::new(),
            dead_streaks: Vec::new(),

            stdout: HideCursor::from(
                stdout()
                    .into_alternate_screen()
                    .expect("Failed to create alternative screen"),
            ),

            xoshiro256p: Xoshiro256pState::new(0xdeadbeef),
        };

        printer.init().map(|_| printer)
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

            if self.streaks[streak_idx].is_dead() {
                self.dead_streaks.push(idx);
            } else {
                // advance streak (wrapping)
                self.streaks[streak_idx].extend(self.size.1);
            }
        }

        // remove dead streaks from intialized and add them to pending
        while let Some(idx) = self.dead_streaks.pop() {
            self.pending_streaks
                .push(self.initialized_streaks.remove(idx));
        }

        // initialize streaks after re-initing vector or removing streak
        if let Some(idx) = self.pending_streaks.pop() {
            let pos = (
                // col determined by shuffled idxs vector + transmute for skipped cols
                (idx as u16 * 2) + 1,
                // random row
                self.xoshiro256p.next() as u16 % self.size.1,
            );

            self.streaks[idx].init(pos, self.xoshiro256p.next());
            self.initialized_streaks.push(idx);
        }

        self.stdout.flush()?;

        Ok(())
    }
}

/// Private methods
impl Printer {
    fn init(&mut self) -> IOResult {
        self.fetch_size()?;
        self.reinit()
    }

    fn reinit(&mut self) -> IOResult {
        self.set_background()?;

        // only fill every other column and skip the first and last one
        self.streaks
            .resize_with((self.size.0 as usize - 2) / 2, Default::default);

        self.pending_streaks = (0..(self.size.0 as usize - 2) / 2).collect();
        self.initialized_streaks.clear();

        // "shuffle" number idxs by permutating elements
        for _ in 0..self.pending_streaks.len() {
            let idx = self.xoshiro256p.next() as usize % self.pending_streaks.len();
            let jdx = self.xoshiro256p.next() as usize % self.pending_streaks.len();

            self.pending_streaks.swap(idx, jdx);
        }

        Ok(())
    }

    fn fetch_size(&mut self) -> IOResult {
        let old_size = self.size;
        self.size = terminal_size()?;
        self.size_changed = self.size != old_size;

        Ok(())
    }

    fn clear_pos(&mut self, row: u16, col: u16) -> IOResult {
        write!(&mut self.stdout, "{} ", cursor::Goto(col + 1, row + 1))
    }

    fn set_pos(&mut self, row: u16, col: u16, c: char, rgb: Rgb) -> IOResult {
        let pos = cursor::Goto(col + 1, row + 1);
        write!(&mut self.stdout, "{pos}{}{c}", color::Fg(rgb))
    }

    fn set_background(&mut self) -> IOResult {
        let pos = cursor::Goto(1, 1);
        let fill = " ".repeat(self.size.0 as usize * self.size.1 as usize);
        let background = color::Bg(Rgb(20, 20, 20));

        write!(&mut self.stdout, "{pos}{background}{fill}")
    }

    fn wrapping_row_div(&self, lhs: u16, rhs: u16) -> u16 {
        if lhs >= rhs {
            lhs - rhs
        } else {
            self.size.1 + lhs - rhs
        }
    }
}
