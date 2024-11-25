#![feature(test)] // enable benchmarks

use rand::prelude::*;
use rayon::prelude::*;

/// calculates the number of turns lost out of `tries` attempts
fn calculate_attempt(tries: u32) -> u32 {
    let mut rng = rand::thread_rng();
    let mut turns_lost: u32 = 0;
    for _ in 0..(tries / 16) {
        let moves: u32 = calculate_moves(16, &mut rng);
        turns_lost += moves.count_ones();
    }
    turns_lost + calculate_moves(tries % 16, &mut rng).count_ones()
}

/// generates `out_of` moves from one call of rand. Since there are 32 bits generated and 2 bits are needed for 1/4 odds
/// the max number of moves is 16.
fn calculate_moves(out_of: u32, rng: &mut ThreadRng) -> u32 {
    assert!(out_of <= 16);
    let moves: u32 = rng.gen();
    (moves >> out_of) & (moves & ((1 << out_of) - 1))
}

/// Uses rayon to calculate iterations in parallel
fn calculate_odds(iterations: usize, moves_per_try: u32) -> u32 {
    (0..iterations).into_par_iter().map(|_| {calculate_attempt(moves_per_try)}).max().expect("`iterations` must not be zero")
}

/// run with `cargo run --release` for optimizations
fn main() {
    println!("{:?}", calculate_odds(1_000_000_000, 231));
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::*;

    #[bench]
    /// Results on my machine:
    /// Ryzen 9 7900
    /// 12 cores
    /// 24 threads
    /// base clock 3.7 Ghz
    /// boost clock 5.4 Ghz
    /// 2,104,490.00 ns/iter (+/- 56,462.25)
    /// or 0.0021 sec/iter (+/- 0.000056)
    fn bench(bencher: &mut Bencher) {
        bencher.iter(|| calculate_odds(1_000_000_000, 231))
    }
}
