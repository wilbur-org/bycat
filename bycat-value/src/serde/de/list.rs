use crate::List;
use core::fmt;
use core::marker::PhantomData;
use serde::de;

struct ListVisitor<T>(PhantomData<T>);

impl<T> ListVisitor<T> {
    fn new() -> Self {
        ListVisitor(PhantomData)
    }
}

impl<'de, T> de::Visitor<'de> for ListVisitor<T>
where
    T: Clone + de::Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = List<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut list = List::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(value) = seq.next_element::<T>()? {
            list.push(value);
        }

        Ok(list)
    }
}

impl<'de, T> de::Deserialize<'de> for List<T>
where
    T: Clone + de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListVisitor::new())
    }
}
