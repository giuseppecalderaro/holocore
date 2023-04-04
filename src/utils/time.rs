use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Utc};

pub fn now() -> (i64, u32) {
    let now = Utc::now();
    (now.timestamp(), now.timestamp_subsec_nanos())
}

pub fn now_millis() -> i64 {
    let now = Utc::now();
    now.timestamp_millis()
}

pub fn now_micros() -> i64 {
    let now = Utc::now();
    now.timestamp_micros()
}

pub fn now_nanos() -> i64 {
    let now = Utc::now();
    now.timestamp_nanos()
}

pub fn datetime(year: i32, month: u32, day: u32, hour: u32, minutes: u32, seconds: u32) -> String {
    let date = NaiveDate::from_ymd(year, month, day);
    let time = NaiveTime::from_hms(hour, minutes, seconds);
    let datetime = NaiveDateTime::new(date, time);
    datetime.format("%Y%m%dT%H%M%S").to_string()
}

pub fn datetime_from_ts(ts: (i64, u32)) -> String {
    let datetime = NaiveDateTime::from_timestamp(ts.0, ts.1);
    datetime.format("%Y%m%dT%H%M%S").to_string()
}

pub fn date(year: i32, month: u32, day: u32) -> String {
    let date = NaiveDate::from_ymd(year, month, day);
    date.format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use crate::utils::time::{ date, datetime, datetime_from_ts };

    #[test]
    fn test_datetime() {
        let new_datetime = datetime(2022, 9, 21, 19, 39, 30);
        assert_eq!(new_datetime, "20220921T193930");
    }

    #[test]
    fn test_datetime_from_timestamp() {
        let new_datetime = datetime_from_ts((1663790343, 936116000));
        assert_eq!(new_datetime, "20220921T195903");
    }

    #[test]
    fn test_date() {
        let new_date = date(2022, 9, 21);
        assert_eq!(new_date, "20220921");
    }
}
