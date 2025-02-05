use std::ops::Index;

use crate::xoshiro256p::Xoshiro256pState;

const CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[derive(Default, Clone)]
pub struct NumberStreak {
    /// Alphanumeric characters of the streak
    vals: Vec<char>,
    val_tail: usize,
    val_head: usize,

    /// Position of the streak head (col, pos)
    head_pos: (u16, u16),

    /// Random number generator
    xoshiro256p: Xoshiro256pState,
}

impl NumberStreak {
    pub fn init(&mut self, head_pos: (u16, u16), max_len: usize, seed: u64) {
        self.vals.resize(max_len, ' ');

        self.val_tail = 0;
        self.val_head = 0;

        self.head_pos = head_pos;

        self.xoshiro256p = Xoshiro256pState::new(seed);
    }

    pub fn extend(&mut self, row_limit: u16) {
        self.head_pos.1 += 1;
        self.head_pos.1 %= row_limit;

        if self.val_tail == self.vals.len() {
            // if death process started, continue every time iter is called
            if self.val_head != 0 || self.xoshiro256p.next() & 0x3ff >= 1015 {
                self.vals[self.val_head] = ' ';
                self.val_head += 1;
            }

            return;
        }

        self.vals[self.val_tail] = CHARS[self.xoshiro256p.next() as usize % CHARS.len()];
        self.val_tail += 1;
    }

    pub fn col(&self) -> u16 {
        self.head_pos.0
    }

    pub fn row(&self) -> u16 {
        self.head_pos.1
    }

    pub fn len(&self) -> u16 {
        self.val_tail as u16
    }

    pub fn is_dead(&self) -> bool {
        // this implies that the entire vals array has been replaced with spaces
        // "fading out" the streak
        self.vals.len() == self.val_head
    }
}

impl Index<usize> for NumberStreak {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vals[index]
    }
}
