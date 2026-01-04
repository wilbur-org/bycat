use crate::Map;
use core::fmt;
use core::marker::PhantomData;
use serde::de;

struct MapVisitor<K, V>(PhantomData<K>, PhantomData<V>);

impl<K, V> MapVisitor<K, V> {
    fn new() -> Self {
        MapVisitor(PhantomData, PhantomData)
    }
}

// This is the trait that Deserializers are going to be driving. There
// is one method for each type of data that our type knows how to
// deserialize from. There are many other methods that are not
// implemented here, for example deserializing from integers or strings.
// By default those methods will return an error, which makes sense
// because we cannot deserialize a MyMap from an integer or string.
impl<'de, K, V> de::Visitor<'de> for MapVisitor<K, V>
where
    K: Clone + de::Deserialize<'de> + Ord,
    V: de::Deserialize<'de> + Clone,
{
    // The type that our Visitor is going to produce.
    type Value = Map<K, V>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let mut map = Map::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry::<K, V>()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<'de, K, V> de::Deserialize<'de> for Map<K, V>
where
    K: Clone + de::Deserialize<'de> + Ord,
    V: de::Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(MapVisitor::new())
    }
}
