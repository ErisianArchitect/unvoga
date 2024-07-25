use std::{sync::{LazyLock, OnceLock}, time::Instant};

use hashbrown::HashMap;
use itertools::Itertools;

struct BlockRegistry {
    blocks: HashMap<String, u32>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self {
            blocks: {
                let mut blocks = HashMap::new();
                for i in 0..1000 {
                    blocks.insert(format!("block_{i:03}"), i);
                }
                blocks
            }
        }
    }

    pub fn get_value(&self, key: &str) -> u32 {
        if let Some(&value) = self.blocks.get(key) {
            value
        } else {
            0
        }
    }
}

static mut LAZY_LOCK: LazyLock<BlockRegistry> = LazyLock::new(BlockRegistry::new);
static mut ONCE_LOCK: OnceLock<BlockRegistry> = OnceLock::new();

fn method_lazy<S: AsRef<str>>(keys: &[&S]) -> u128 {
    keys.iter().map(|&key| {
        unsafe {
            LAZY_LOCK.get_value(key.as_ref()) as u128
        }
    }).sum()
}

fn method_once<S: AsRef<str>>(keys: &[&S]) -> u128 {
    keys.iter().map(|&key| {
        unsafe {
            let once = ONCE_LOCK.get().expect("Failed to get OnceLock");
            once.get_value(key.as_ref()) as u128
        }
    }).sum()
}

fn main() {
    unsafe {
        let _ = ONCE_LOCK.set(BlockRegistry::new());
    }
    let entry_strings = (0..1000).map(|i| format!("block_{i:03}")).collect_vec();
    let keys = (0..1000000usize)
        .map(|i| i.rem_euclid(entry_strings.len()))
        .map(|i| &entry_strings[i])
        .collect_vec();
    let timer_lazy = Instant::now();
    let result_lazy = method_lazy(&keys);
    let elapsed_lazy = timer_lazy.elapsed();
    let timer_once = Instant::now();
    let result_once = method_once(&keys);
    let elapsed_once = timer_once.elapsed();
    println!("Once: {}, {result_once}", elapsed_once.as_secs_f32());
    println!("Lazy: {}, {result_lazy}", elapsed_lazy.as_secs_f32());

    let timer_once = Instant::now();
    let result_once = method_once(&keys);
    let elapsed_once = timer_once.elapsed();
    let timer_lazy = Instant::now();
    let result_lazy = method_lazy(&keys);
    let elapsed_lazy = timer_lazy.elapsed();
    println!("Once: {}, {result_once}", elapsed_once.as_secs_f32());
    println!("Lazy: {}, {result_lazy}", elapsed_lazy.as_secs_f32());
}