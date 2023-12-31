fn parse_datetime_from_str(s: &str) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
	let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S");
	if dt.is_ok() {
		return dt;
	}

	let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M");
	if dt.is_ok() {
		return dt;
	}

	match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
		Ok(date) => Ok(date.and_time(chrono::NaiveTime::default())),
		Err(e) => Err(e),
	}
}

pub fn deserialize_string_to_naivedatetime<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error> {
	let s: String = serde::Deserialize::deserialize(deserializer)?;
	parse_datetime_from_str(&s).map_err(serde::de::Error::custom)
}

pub fn serialize_naivedatetime_to_i64<S: serde::Serializer>(
	value: &chrono::NaiveDateTime,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	serializer.serialize_i64(value.timestamp())
}

pub fn drop_trailing_slash(s: &mut String) {
	if s.ends_with("/") {
		s.pop();
	}
}
