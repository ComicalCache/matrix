/*
 * xoshiro256+ implementation
 * https://en.wikipedia.org/wiki/Xorshift#xoshiro256+
 */

fn rol64(x: u64, k: u32) -> u64 {
    (x << k) | (x >> (64 - k))
}

#[derive(Default, Clone, Copy)]
pub struct Xoshiro256pState([u64; 4]);

impl Xoshiro256pState {
    pub fn new(seed: u64) -> Self {
        let mut seed: Splitmix64State = seed;

        let mut state = Xoshiro256pState([0; 4]);

        state.0[0] = splitmix64(&mut seed);
        state.0[1] = splitmix64(&mut seed);
        state.0[2] = splitmix64(&mut seed);
        state.0[3] = splitmix64(&mut seed);

        state
    }

    pub fn next(&mut self) -> u64 {
        let res = self.0[0].wrapping_add(self.0[3]);

        let tmp = self.0[1] << 17;

        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];

        self.0[2] ^= tmp;
        self.0[3] = rol64(self.0[3], 45);

        res
    }
}

/*
 * splitmix64 used for initialization
 * https://en.wikipedia.org/wiki/Xorshift#Initialization
 */

type Splitmix64State = u64;

fn splitmix64(state: &mut Splitmix64State) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);

    let mut result = *state;
    result = (result ^ (result >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94d049bb133111eb);
    result ^ (result >> 31)
}
