#![allow(unused)]
use unvoga::core::math::bit::*;

const ITERATIONS: u64 = 100000000;
const LOOPS: u32 = 10;

fn invert_bit_old<I: ShiftIndex>(value: u64, index: I) -> u64 {
    let bit = value.get_bit(index);
    value.set_bit(index, !bit)
}

fn main() {
    let (time1, result1, counter1) = std::hint::black_box(method1());
    let (time2, result2, counter2) = std::hint::black_box(method2());
    println!("  Optimized: {time1}");
    println!("Unoptimized: {time2}");
    println!("Result1: {result1}");
    println!("Result2: {result2}");
    println!("Counter1: {counter1}");
    println!("Counter2: {counter2}");
}

fn method1() -> (f64, u64, u64) {
    let mut no_opt = 0u64;
    let mut accum = 0.0;
    let mut counter = 0u64;
    for li in 0..LOOPS {
        let start_time = std::time::Instant::now();
        for i in 0..ITERATIONS {
            no_opt = counter.invert_bit(i % 64);
            counter += 1;
        }
        let elapsed = start_time.elapsed();
        accum += elapsed.as_secs_f64();
    }
    let average = accum / LOOPS as f64;
    (average, no_opt, counter)
}

fn method2() -> (f64, u64, u64) {
    let mut no_opt = 0u64;
    let mut accum = 0.0;
    let mut counter = 0u64;
    for li in 0..LOOPS {
        let start_time = std::time::Instant::now();
        for i in 0..ITERATIONS {
            no_opt = invert_bit_old(counter, i % 64);
            counter += 1;
        }
        let elapsed = start_time.elapsed();
        accum += elapsed.as_secs_f64();
    }
    let average = accum / LOOPS as f64;
    (average, no_opt, counter)
}