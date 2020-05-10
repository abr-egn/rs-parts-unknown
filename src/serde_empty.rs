use serde::{
    Deserializer, Serializer,
    de::IgnoredAny,
    ser::SerializeStruct,
};
pub fn serialize<S: Serializer>(s: S) -> Result<S::Ok, S::Error> {
    s.serialize_struct("UNUSED_NAME", 0)?.end()
}
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<(), D::Error> {
    //d.deserialize_any(IgnoredAny).map(|_| ())
    d.deserialize_struct("UNUSED_NAME", &[], IgnoredAny).map(|_| ())
}