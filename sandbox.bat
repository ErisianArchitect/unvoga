@echo OFF
set RUST_BACKTRACE=FULL
cargo run --release --bin sandbox %*