use bitwise::word::*;
use byteorder::{BigEndian, ByteOrder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitfield([u64; 4]);

impl Serialize for Bitfield {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut v = vec![0u8; 4 * 8];
        // Big endian ordering, to match go
        BigEndian::write_u64(&mut v[..8], self.0[3]);
        BigEndian::write_u64(&mut v[8..16], self.0[2]);
        BigEndian::write_u64(&mut v[16..24], self.0[1]);
        BigEndian::write_u64(&mut v[24..], self.0[0]);

        let byte_buf = serde_bytes::ByteBuf::from(v);
        byte_buf.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Bitfield {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut res = Bitfield::zero();
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        res.0[3] = BigEndian::read_u64(&bytes[..8]);
        res.0[2] = BigEndian::read_u64(&bytes[8..16]);
        res.0[1] = BigEndian::read_u64(&bytes[16..24]);
        res.0[0] = BigEndian::read_u64(&bytes[24..]);

        Ok(res)
    }
}

impl Default for Bitfield {
    fn default() -> Self {
        Bitfield::zero()
    }
}

impl Bitfield {
    pub fn clear_bit(&mut self, idx: u8) {
        let ai = idx / 64;
        let bi = idx % 64;
        self.0[ai as usize] = self.0[ai as usize].clear_bit(bi as u32);
    }

    pub fn test_bit(&self, idx: u8) -> bool {
        let ai = idx / 64;
        let bi = idx % 64;

        self.0[ai as usize].test_bit(bi as u32)
    }

    pub fn set_bit(&mut self, idx: u8) {
        let ai = idx / 64;
        let bi = idx % 64;

        self.0[ai as usize] = self.0[ai as usize].set_bit(bi as u32);
    }

    pub fn count_ones(&self) -> usize {
        self.0.iter().map(|a| a.count_ones() as usize).sum()
    }

    pub fn and(self, other: &Self) -> Self {
        Bitfield([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }

    pub fn zero() -> Self {
        Bitfield([0, 0, 0, 0])
    }

    pub fn set_bits_le(self, bit: u8) -> Self {
        if bit == 0 {
            return self;
        }
        self.set_bits_leq(bit - 1)
    }

    pub fn set_bits_leq(mut self, bit: u8) -> Self {
        if bit < 64 {
            self.0[0] = self.0[0].set_bits_leq(bit);
        } else if bit < 128 {
            self.0[0] = std::u64::MAX;
            self.0[1] = self.0[1].set_bits_leq(bit as u32 - 64);
        } else if bit < 192 {
            self.0[0] = std::u64::MAX;
            self.0[1] = std::u64::MAX;
            self.0[2] = self.0[2].set_bits_leq(bit as u32 - 128);
        } else {
            self.0[0] = std::u64::MAX;
            self.0[1] = std::u64::MAX;
            self.0[2] = std::u64::MAX;
            self.0[3] = self.0[3].set_bits_leq(bit as u32 - 192);
        }

        self
    }
}

impl std::fmt::Binary for Bitfield {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let val = self.0;

        write!(f, "{:b}_{:b}_{:b}_{:b}", val[0], val[1], val[2], val[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitfield() {
        let mut b = Bitfield::zero();
        b.set_bit(8);
        b.set_bit(18);
        b.set_bit(92);
        b.set_bit(255);
        println!("{:?}", &b);
        assert!(b.test_bit(8));
        assert!(b.test_bit(18));
        assert!(!b.test_bit(19));
        assert!(b.test_bit(92));
        assert!(!b.test_bit(95));
        assert!(b.test_bit(255));

        b.clear_bit(18);
        assert!(!b.test_bit(18));
    }

}
