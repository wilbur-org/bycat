use crate::Map;

impl serde::ser::Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.entries).serialize(serializer)
    }
}
