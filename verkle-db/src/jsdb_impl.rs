use wasm_bindgen::prelude::*;

use crate::{BareMetalDiskDb, BareMetalKVDb};


#[wasm_bindgen(module="./db.js")]
extern "C" {
    type jsKVDB;

    #[wasm_bindgen(constructor)]
    fn from_path() -> jsDB;

    #[wasm_bindgen(constructor)]
    fn new() -> jsKVDB;

    #[wasm_bindgen(method)]
    fn fetch(this: &jsKVDB, key: [u8]) -> Option<Vec<u8>>;

    #[wasm_bindgen(method)]
    fn batch_put(this: &jsKVDB, key: &[u8], val: &[u8]);

    #[wasm_bindgen(method)]
    fn write(this: &jsKVDB, keys: <Vec<u8>>, val: <Vec<u8>>);
}

pub struct WriteBatcher {
    keys: Vec<u8>,
    vals: Vec<u8>
}

impl BareMetalDiskDb for jsKVDB {
    fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self {
        // use rusty_leveldb::{CompressionType, Options};
        // let mut opt = Options::default();
        // opt.compression_type = CompressionType::CompressionSnappy;
        let db = jsKVDB::from_path();
        db
    }

    const DEFAULT_PATH: &'static str = "./db/verkle_db";
}


impl BareMetalKVDb for jsKVDB {
    fn fetch(&self, key: &[u8]) -> Option<Vec<u8>> {
        let vector = self.fetch(key);
        vector
    }
    // Create a database given the default path
    fn new() -> Self {
        let db = jsKVDB::from_path();
        db
    }
}

use crate::{BatchDB, BatchWriter};

use WriteBatcher as WriteBatch;
impl BatchWriter for WriteBatcher {
    fn new() -> Self {
        let batchWriter = jsKVDB::new();
        batchWriter
    }

    fn batch_put(self, key: &[u8], val: &[u8]) {
        self.batch_put(key, val)
    }
}

use jsKVDB as DB;
impl BatchDB for DB {
    type BatchWrite = WriteBatch;

    fn flush(self, batch: Self::BatchWrite) {
        self.write(batch);
    }
}
