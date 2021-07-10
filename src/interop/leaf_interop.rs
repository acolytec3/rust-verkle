#[cfg(test)]
mod test {
    use ark_ff::{BigInteger, PrimeField};

    use crate::{trie::node::leaf::LeafNode, Key, Value};

    // Key  =  0000000000000000000000000000000000000000000000000000000000000000
    // Value = 0000000000000000000000000000000000000000000000000000000000000000
    #[test]
    pub fn k0v0() {
        let leaf = LeafNode::new(Key::zero(), Value::zero());
        let hash = leaf.hash().to_hex();
        assert_eq!(
            hash,
            "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
        )
    }
    // Key  =  0000000000000000000000000000000000000000000000000000000000000001
    // Value = 0000000000000000000000000000000000000000000000000000000000000000
    #[test]
    pub fn k1v0() {
        let leaf = LeafNode::new(Key::one(), Value::zero());
        let hash = leaf.hash().to_hex();
        assert_eq!(
            hash,
            "58e8f2a1f78f0a591feb75aebecaaa81076e4290894b1c445cc32953604db089"
        )
    }
    // Key  =  0000000000000000000000000000000000000000000000000000000000000001
    // Value = 0000000000000000000000000000000000000000000000000000000000000001
    #[test]
    pub fn k1v1() {
        let leaf = LeafNode::new(Key::one(), Value::one());
        let hash = leaf.hash().to_hex();
        assert_eq!(
            hash,
            "c3c3a46684c07d12a9c238787df3049a6f258e7af203e5ddb66a8bd66637e108"
        )
    }
    // Key  =  0000000000000000000000000000000000000000000000000000000000000001
    // Value = 0000000000000000000000000000000000000000000000000000000000000001
    // See `values[i] = int.from_bytes(node[i]["hash"], "little") % MODULUS`
    #[test]
    pub fn k1v1_fr() {
        let leaf = LeafNode::new(Key::one(), Value::one());

        let leaf_bytes = leaf.hash().to_fr().into_repr().to_bytes_be();
        assert_eq!(
            "08e13766d68b6ab6dde503f27a8e256f9a04f37d7838c2a9127dc08466a4c3c3",
            hex::encode(&leaf_bytes)
        )
    }
}
