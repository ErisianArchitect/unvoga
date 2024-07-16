use chrono::*;

use crate::prelude::{Readable, Writeable};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Writeable for Timestamp {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> crate::prelude::VoxelResult<u64> {
        self.0.write_to(writer)
    }
}

impl Readable for Timestamp {
    fn read_from<R: std::io::Read>(reader: &mut R) -> crate::prelude::VoxelResult<Self> {
        Ok(Timestamp(i64::read_from(reader)?))
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