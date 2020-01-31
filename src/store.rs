use std::collections::HashMap;
use std::sync::RwLock;

use cid::Cid;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::Digest;

use crate::error::Result;
use crate::tagged_cid::TaggedCid;

pub trait Store: std::fmt::Debug {
    fn insert<B: Serialize>(&self, block: &B) -> Result<TaggedCid>;
    fn get<B: DeserializeOwned>(&self, cid: &TaggedCid) -> Result<Option<B>>;
    fn get_bytes(&self, cid: &TaggedCid) -> Result<Option<Vec<u8>>>;
}

#[derive(Default, Debug)]
pub struct MemoryStore {
    data: RwLock<HashMap<TaggedCid, Vec<u8>>>,
}

impl Store for MemoryStore {
    fn insert<B: Serialize>(&self, block: &B) -> Result<TaggedCid> {
        let (c, bytes) = serialize_sha256(block)?;
        self.data.write().unwrap().insert(c.clone(), bytes);

        Ok(c)
    }

    fn get<B: DeserializeOwned>(&self, cid: &TaggedCid) -> Result<Option<B>> {
        match self.data.read().unwrap().get(cid) {
            Some(ref bytes) => {
                let obj = serde_cbor::from_slice(bytes)?;
                Ok(Some(obj))
            }
            None => Ok(None),
        }
    }

    fn get_bytes(&self, cid: &TaggedCid) -> Result<Option<Vec<u8>>> {
        Ok(self.data.read().unwrap().get(cid).map(|v| v.to_vec()))
    }
}

fn _serialize_blake2b<D: Serialize>(data: &D) -> Result<(TaggedCid, Vec<u8>)> {
    let bytes = serde_cbor::to_vec(data)?;

    // TODO: fix cid and multihash!!!
    let h = blake2b_simd::blake2b(&bytes);
    let code = multihash::Hash::Blake2b.code();
    let size = multihash::Hash::Blake2b.size();
    let mut hash = vec![0; size as usize + 2];
    hash[0] = code;
    hash[1] = size;
    hash[2..].copy_from_slice(h.as_ref());

    let c = Cid {
        version: cid::Version::V1,
        codec: cid::Codec::DagCBOR,
        hash,
    };

    Ok((c.into(), bytes))
}

fn serialize_sha256<D: Serialize>(data: &D) -> Result<(TaggedCid, Vec<u8>)> {
    let bytes = serde_cbor::to_vec(&serde_cbor::value::to_value(data.clone())?)?;

    // TODO: fix cid and multihash!!!
    let h = sha2::Sha256::digest(&bytes);
    let code = multihash::Hash::SHA2256.code();
    let size = multihash::Hash::SHA2256.size();
    let mut hash = vec![0; size as usize + 2];
    hash[0] = code;
    hash[1] = size;
    hash[2..].copy_from_slice(h.as_ref());

    let c = Cid {
        version: cid::Version::V1,
        codec: cid::Codec::DagCBOR,
        hash,
    };

    Ok((c.into(), bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[test]
    fn test_memory() {
        let store = MemoryStore::default();

        let c1 = store.insert(&("hello".to_string(), 3)).unwrap();
        let back = store.get(&c1).unwrap();
        assert_eq!(back, Some(("hello".to_string(), 3)));
    }

    #[test]
    fn test_memory_interop() {
        let store = MemoryStore::default();

        let mut thingy1 = HashMap::new();
        thingy1.insert("cat".to_string(), "dog".to_string());

        let c1 = store.insert(&thingy1).unwrap();

        assert_eq!(
            c1,
            Cid::from("zdpuAqYjGuvUBhcmyFhHjh9mZbBW5MYLD2eUcXTWqmj73dHXD")
                .unwrap()
                .into()
        );

        #[derive(Debug, Serialize, Deserialize)]
        struct Thingy2 {
            one: TaggedCid,
            foo: String,
        }

        let thingy2 = Thingy2 {
            one: c1.clone().into(),
            foo: "bar".into(),
        };

        let c2 = store.insert(&thingy2).unwrap();
        println!("{}", hex::encode(store.get_bytes(&c2).unwrap().unwrap()));

        assert_eq!(
            c2,
            Cid::from("zdpuAt1cw4ZvvLnXL9KFbEkM3vXibtwiJek8d3o4h1fPkEgMX")
                .unwrap()
                .into()
        );

        let mut hamt: crate::hamt::Hamt<String, Thingy2, _> = crate::hamt::Hamt::new(&store);
        hamt.insert("cat".to_string(), thingy2);

        let c3 = store.insert(&hamt).unwrap();
        println!(
            "c3: {}",
            hex::encode(store.get_bytes(&c3).unwrap().unwrap())
        );
        println!("{:#?}", &hamt);

        // Not quite there yet
        // assert_eq!(
        //     c3,
        //     Cid::from("zdpuApTKRtVAtwquN7f3A5bZBXnsLkmpLQfF7CVAeGDbkL5Zo")
        //         .unwrap()
        //         .into()
        // );
    }
}
