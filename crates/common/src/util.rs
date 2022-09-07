use std::str::FromStr;

use chrono::{DateTime, Utc, TimeZone, Date, NaiveDate};
use serde::{Serializer, Deserializer, Deserialize, Serialize};


pub const FILE_SIZE_IDENTIFIERS: [&str; 4] = ["B", "KB", "MB", "GB"];





pub fn file_size_bytes_to_readable_string(value: i64) -> String {
    let mut size = value as f64;

    // 1024

    let mut index = 0;
    while size > 1024.0 && index != 3 {
        size /= 1024.0;
        index += 1;
    }

    if index + 1 == FILE_SIZE_IDENTIFIERS.len() {
        format!("{}{}", (size * 100.0).floor() / 100.0, FILE_SIZE_IDENTIFIERS[index])
    } else {
        format!("{}{}", size.floor(), FILE_SIZE_IDENTIFIERS[index])
    }
}



/// Parse a string which is "{number}-{description}"
pub fn parse_num_description_string<D: FromStr>(value: &str) -> std::result::Result<D, D::Err> {
    let value = value.split_once('-')
        .map(|v| v.0)
        .unwrap_or(value);

    D::from_str(value)
}



// Serde

pub fn serialize_datetime<S>(value: &DateTime<Utc>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
    s.serialize_i64(value.timestamp_millis())
}

pub fn serialize_datetime_opt<S>(value: &Option<DateTime<Utc>>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
    match value {
        Some(v) => s.serialize_i64(v.timestamp_millis()),
        None => s.serialize_none()
    }
}

pub fn serialize_datetime_opt_opt<S>(value: &Option<Option<DateTime<Utc>>>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
    value.map(|v| v.map(|v| v.timestamp_millis())).serialize(s)
}


pub fn deserialize_datetime<'de, D>(value: D) -> std::result::Result<DateTime<Utc>, D::Error> where D: Deserializer<'de> {
    Ok(Utc.timestamp_millis(i64::deserialize(value)?))
}

pub fn deserialize_datetime_opt<'de, D>(value: D) -> std::result::Result<Option<DateTime<Utc>>, D::Error> where D: Deserializer<'de> {
    if let Some(v) = Option::<i64>::deserialize(value)? {
        Ok(Some(Utc.timestamp_millis(v)))
    } else {
        Ok(None)
    }
}

pub fn deserialize_datetime_opt_opt<'de, D>(value: D) -> std::result::Result<Option<Option<DateTime<Utc>>>, D::Error> where D: Deserializer<'de> {
    if let Some(v) = Option::<Option<i64>>::deserialize(value)? {
        Ok(Some(v.map(|v| Utc.timestamp_millis(v))))
    } else {
        Ok(None)
    }
}

// Date

pub fn serialize_date<S>(value: &Date<Utc>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
    s.serialize_i64(value.and_hms(0, 0, 0).timestamp())
}

pub fn serialize_naivedate_opt<S>(value: &Option<NaiveDate>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
    match value {
        Some(v) => s.serialize_i64(v.and_hms(0, 0, 0).timestamp()),
        None => s.serialize_none()
    }
}


pub fn deserialize_date<'de, D>(value: D) -> std::result::Result<Date<Utc>, D::Error> where D: Deserializer<'de> {
    Ok(Utc.timestamp(i64::deserialize(value)?, 0).date())
}

pub fn deserialize_naivedate_opt<'de, D>(value: D) -> std::result::Result<Option<NaiveDate>, D::Error> where D: Deserializer<'de> {
    if let Some(v) = Option::<i64>::deserialize(value)? {
        Ok(Some(Utc.timestamp(v, 0).date_naive()))
    } else {
        Ok(None)
    }
}


pub fn des_if_opt_str_not_empty<'de, D>(value: D) -> std::result::Result<Option<String>, D::Error> where D: Deserializer<'de> {
    Ok(Option::<String>::deserialize(value)?.filter(|v| !v.trim().is_empty()))
}