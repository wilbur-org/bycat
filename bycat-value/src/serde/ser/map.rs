use crate::Map;

impl<K, V> serde::ser::Serialize for Map<K, V>
where
    K: serde::ser::Serialize,
    V: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.entries).serialize(serializer)
    }
}
