use serde::{
    Deserializer, Serializer,
    de::IgnoredAny,
};
pub fn serialize<S: Serializer>(s: S) -> Result<S::Ok, S::Error> {
    s.serialize_bool(true)
}
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<(), D::Error> {
    d.deserialize_any(IgnoredAny).map(|_| ())
}