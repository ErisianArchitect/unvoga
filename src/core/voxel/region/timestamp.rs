use chrono::*;

pub struct Timestamp(i64);

impl Timestamp {
    #[inline(always)]
    pub const fn new(timestamp: i64) -> Self {
        Self(timestamp)
    }

    #[inline(always)]
    pub fn utc_now() -> Self {
        Self(Utc::now().timestamp())
    }

    /// Gets the UNIX UTC timestamp.
    #[inline(always)]
    pub const fn timestamp(self) -> i64 {
        self.0
    }

    /// Gets the [Utc] [DateTime].
    #[inline(always)]
    pub fn time(self) -> DateTime<Utc> {
        chrono::DateTime::from_timestamp(self.0, 0).expect("Timestamp was invalid.")
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    #[test]
    fn timestamp_test() {
        let ts = Timestamp::utc_now();
        let dt = ts.time();
        println!("{dt}");
    }
}