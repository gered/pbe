pub fn deserialize_naivedate<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<chrono::NaiveDate, D::Error> {
	let s: String = serde::Deserialize::deserialize(deserializer)?;
	chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

pub fn serialize_naivedate<S: serde::Serializer>(value: &chrono::NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(&value.to_string())
}

pub fn safe_subslice<T>(slice: &[T], start: usize, count: usize) -> Option<&[T]> {
	if start >= slice.len() {
		return None;
	}
	let end = std::cmp::min(start + count - 1, slice.len() - 1);
	Some(&slice[start..=end])
}
