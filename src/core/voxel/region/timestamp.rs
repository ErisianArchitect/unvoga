use chrono::TimeZone;

pub struct Timestamp(i64);

impl Timestamp {
    #[inline(always)]
    pub fn utc_now() -> Self {
        Self(chrono::Utc::now().timestamp_micros())
    }
}