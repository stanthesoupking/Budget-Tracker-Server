use std::time::SystemTime;

use chrono::{Date, DateTime, Utc, FixedOffset, ParseResult, Datelike, Timelike};

pub fn get_current_date() -> String {
    let now = Utc::now();

    format!("{}-{}-{}", now.year(), now.month(), now.day())
}

pub fn get_current_date_time() -> String {
    let now = Utc::now();

    format!("{}-{}-{} {}:{}", now.year(), now.month(), now.day(),
        now.hour(), now.minute())
}

pub fn from_sqlite_date(sdate: &String) -> ParseResult<DateTime<FixedOffset>> {    
    DateTime::parse_from_str(
                &format!("{} 00:00:00 +0000", sdate),
                "%Y-%-m-%-d %T %z"
    )
}

pub fn to_sqlite_date(cdate: &DateTime<FixedOffset>) -> String {
    let mut sdate = String::with_capacity(11);

    format!("{}-{}-{}", cdate.year(), cdate.month(), cdate.day())
}