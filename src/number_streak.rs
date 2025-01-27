use crate::xoshiro256p::Xoshiro256pState;

pub const STREAK_LENGTH: usize = 16;

const CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[derive(Default, Clone, Copy)]
pub struct Pos {
    pub row: u16,
    pub col: u16,
}

#[derive(Default, Clone, Copy)]
pub struct NumberStreak {
    /// Alphanumeric characters of the streak
    vals: [char; STREAK_LENGTH],
    val_tail: usize,
    val_head: usize,

    val_iter_head: usize,

    /// Position of the streak head
    head_pos: Pos,

    /// Random number generator
    xoshiro256p: Xoshiro256pState,
}

impl NumberStreak {
    pub fn init(&mut self, head_pos: Pos, seed: u64) {
        self.val_tail = 0;
        self.val_head = 0;

        self.val_iter_head = 0;

        self.head_pos = head_pos;

        self.xoshiro256p = Xoshiro256pState::new(seed);
    }

    pub fn extend(&mut self, row_limit: u16) {
        self.head_pos.row += 1;
        self.head_pos.row %= row_limit;

        if self.val_tail == STREAK_LENGTH {
            // 0.5% chance of death process starting
            // if death process started, continue every time iter is called
            if self.val_head != 0 || self.xoshiro256p.next() % 100 >= 99 {
                self.vals[self.val_head] = ' ';
                self.val_head += 1;
            }

            return;
        }

        self.vals[self.val_tail] = CHARS[self.xoshiro256p.next() as usize % CHARS.len()];
        self.val_tail += 1;
    }

    pub fn col(&self) -> u16 {
        self.head_pos.col
    }

    pub fn row(&self) -> u16 {
        self.head_pos.row
    }

    pub fn len(&self) -> u16 {
        self.val_tail as u16
    }

    pub fn is_dead(&self) -> bool {
        // this implies that the entire vals array has been replaced with spaces
        // "fading out" the streak
        self.val_tail == self.val_head
    }
}

impl Iterator for NumberStreak {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.val_iter_head >= self.val_tail {
            self.val_iter_head = 0;
            return None;
        }

        let val = self.vals[self.val_iter_head];
        self.val_iter_head += 1;
        Some(val)
    }
}
