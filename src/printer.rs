use std::io::{Stdout, Write, stdout};

use termion::{
    color::{self, Rgb},
    cursor::{self, HideCursor},
    screen::{AlternateScreen, IntoAlternateScreen},
    terminal_size,
};

use colorgrad::{Color, Gradient, GradientBuilder, LinearGradient};

use crate::{IOResult, number_streak::NumberStreak, xoshiro256p::Xoshiro256pState};

pub struct Printer {
    /// Terminal size
    size: (u16, u16),
    size_changed: bool,

    /// Streaks to be drawn
    streaks: Vec<NumberStreak>,
    streak_len: usize,

    pending_streaks: Vec<usize>,
    initialized_streaks: Vec<usize>,
    dead_streaks: Vec<usize>,

    /// Streak colors
    colors: Vec<Rgb>,

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
            streak_len: 0,

            pending_streaks: Vec::new(),
            initialized_streaks: Vec::new(),
            dead_streaks: Vec::new(),

            colors: Vec::new(),

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
            let len = self.streaks[streak_idx].len() as usize;
            for jdx in 0..len {
                let row = self.wrapping_row_div(self.streaks[streak_idx].row(), jdx as u16);
                let char = self.streaks[streak_idx][jdx];
                self.set_pos(row, self.streaks[streak_idx].col(), char, self.colors[jdx])?;
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

            self.streaks[idx].init(pos, self.streak_len, self.xoshiro256p.next());
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

        // adjust streak len for new terminal size
        self.streak_len = (self.size.1 as f32 * 0.25) as usize;

        self.colors.resize(self.streak_len, Rgb(0, 0, 0));

        // regenerate color gradient
        let gradient: LinearGradient = GradientBuilder::new()
            .colors(&[
                Color::from_rgba8(5, 225, 19, 255),
                Color::from_rgba8(52, 61, 49, 255),
            ])
            .build()
            .expect("Failed to build gradient");

        let (min, max) = gradient.domain();
        let delta = max - min;
        let lower = self.streak_len as f32 - 1.;
        for (idx, color) in (0..self.streak_len)
            .map(|i| min + (i as f32 * delta) / lower)
            .map(|t| gradient.at(t).to_rgba8())
            .map(|[r, g, b, _]| Rgb(r, g, b))
            .enumerate()
        {
            self.colors[idx] = color;
        }

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
