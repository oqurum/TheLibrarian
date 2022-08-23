use chrono::{DateTime, Utc, TimeZone, Date};
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


// TODO: Rename
pub fn string_to_upper_case(mut value: String) -> String {
    // Get the first char
    if let Some(v) = value.chars().next() {
        // Uppercase first char
        let first = v.to_uppercase().to_string();

        // Replace first char with uppercase one.
        value.replace_range(0..v.len_utf8(), &first);
    }

    value
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

pub fn deserialize_date<'de, D>(value: D) -> std::result::Result<Date<Utc>, D::Error> where D: Deserializer<'de> {
    Ok(Utc.timestamp(i64::deserialize(value)?, 0).date())
}