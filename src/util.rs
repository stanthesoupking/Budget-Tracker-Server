use std::time::SystemTime;

use chrono::{Date, DateTime, Utc, FixedOffset, ParseResult, Datelike, Timelike, TimeZone, Offset};

pub fn get_now() -> DateTime<FixedOffset> {
    // TODO: Don't hard code this to Tasmanian time
    DateTime::from_utc(Utc::now().naive_utc(), TimeZone::from_offset(&FixedOffset::east(10*60*60)))
}

pub fn get_current_date() -> String {
    let now = get_now();

    format!("{:0>2}-{:0>2}-{:0>2}", now.year(), now.month(), now.day())
}

pub fn get_current_date_time() -> String {
    let now = get_now();

    format!("{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:{:0>2}.{:0>3}", now.year(), now.month(), now.day(),
        now.hour(), now.minute(), now.second(), now.timestamp_subsec_millis())
}

pub fn from_sqlite_date(sdate: &String) -> ParseResult<DateTime<FixedOffset>> {    
    DateTime::parse_from_str(
                &format!("{} 00:00:00 +0000", sdate),
                "%Y-%-m-%-d %T %z"
    )
}

pub fn to_sqlite_date(cdate: &DateTime<FixedOffset>) -> String {
    let mut sdate = String::with_capacity(11);

    format!("{}-{:0>2}-{:0>2}", cdate.year(), cdate.month(), cdate.day())
}