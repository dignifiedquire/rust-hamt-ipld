use cid::Cid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TaggedCid(Cid);

impl TaggedCid {
    fn tag() -> u64 {
        42
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl Serialize for TaggedCid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = self.0.to_bytes();

        // binary multibase is a `0` prefix
        bytes.insert(0, 0);

        let byte_buf = serde_bytes::ByteBuf::from(bytes);
        serde_cbor::EncodeCborTag::new(Self::tag(), &byte_buf).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TaggedCid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapper = serde_cbor::EncodeCborTag::deserialize(deserializer)?;
        if wrapper.tag() != Self::tag() {
            return Err(serde::de::Error::custom(format!(
                "Invalid tag: {}, expected {}",
                wrapper.tag(),
                Self::tag()
            )));
        }
        let bytes: Vec<u8> = wrapper.value();
        // check for binary multibase
        if bytes[0] != 0 {
            return Err(serde::de::Error::custom(format!("invalid link base")));
        }

        Ok(TaggedCid(
            Cid::from(bytes).map_err(serde::de::Error::custom)?,
        ))
    }
}

impl From<Cid> for TaggedCid {
    fn from(c: Cid) -> Self {
        TaggedCid(c)
    }
}

impl AsRef<Cid> for TaggedCid {
    fn as_ref(&self) -> &Cid {
        &self.0
    }
}
