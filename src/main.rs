#![allow(internal_features)]
#![feature(test)] // enable benchmarks

use rand::prelude::*;
use rayon::prelude::*;

/// calculates the number of turns lost out of `tries` attempts
fn calculate_attempt(rng: &mut ThreadRng) -> u64 {
    let mut turns_lost: u64 = 0;
    let mask: u64 = 0x7FFFFFFFFF;
    for mut i in 0u64..4 {
        let mut a: u64 = rng.gen();
        let b: u64 = rng.gen();
        // Assembly is faster for some reason
        #[allow(unused_assignments, reason = "used in asm block")]
        unsafe {
            std::arch::asm!(
                "and {a}, {b}",
                "test {i}, {i}",
                "jne 2f",
                "and {a}, {mask}",
                "2:",
                "popcnt {a}, {a}",
                "add {sum}, {a}",
                a = inout(reg) a,
                b = in(reg) b,
                i = inout(reg) i,
                sum = inout(reg) turns_lost,
                mask = in(reg) mask,
            )
        }
    }
    turns_lost
}

#[inline(never)]
/// Uses rayon to calculate iterations in parallel
fn calculate_odds(iterations: usize) -> u64 {
    if iterations == 0 {
        return 0;
    }
    (0..iterations)
        .into_par_iter()
        .map_init(rand::thread_rng, |rand, _| calculate_attempt(rand))
        .max()
        .unwrap_or(0)
}

/// run with `cargo run --release` for optimizations
fn main() {
    println!("{}", calculate_odds(100_000_000));
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
    /// 1,283,788.75 ns/iter (+/- 38,464.50)
    fn bench(bencher: &mut Bencher) {
        bencher.iter(|| calculate_odds(1_000_000))
    }

    #[test]
    fn assembly_is_zero() {
        let a = 0;
        let b = 1;
        let mut a_is_zero = 0;
        let mut b_is_zero = 0;

        unsafe {
            std::arch::asm!(
                "test {a:e}, {a:e}",
                "jne 2f",
                "mov {a_is_zero:e}, 1",
                "2:",
                "test {b:e}, {b:e}",
                "jne 3f",
                "mov {b_is_zero:e}, 1",
                "3:",
                a = in(reg) a,
                b = in(reg) b,
                a_is_zero = inout(reg) a_is_zero,
                b_is_zero = inout(reg) b_is_zero,
            )
        }

        assert!(a_is_zero == 1);
        assert!(b_is_zero == 0);
    }
}
