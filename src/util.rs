use std::time::SystemTime;

use chrono::Utc;


pub fn get_current_date() -> String {
    Utc::now().to_rfc2822()
}