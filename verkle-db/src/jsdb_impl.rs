use wasm_bindgen::prelude::*;

use crate::{BareMetalDiskDb, BareMetalKVDb};
use js_sys;

#[wasm_bindgen(module="db.js")]
extern "C" {
    pub type jsKVDB;

    #[wasm_bindgen(constructor)]
    pub fn from_path() -> jsKVDB;

    #[wasm_bindgen(constructor)]
    pub fn new() -> jsKVDB;

    #[wasm_bindgen(method)]
    pub fn jsfetch(this: &jsKVDB, key: &[u8]) -> Option<Vec<u8>>;

    #[wasm_bindgen(method)]
    pub fn jsbatch_put(this: &jsKVDB, keys: Vec<js_sys::Uint8Array>, vals: Vec<js_sys::Uint8Array>);

    #[wasm_bindgen(method)]
    pub fn write(this: &jsKVDB, batch: &[u8]);
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
        let vector = self.jsfetch(key);
        vector
    }
    // Create a database given the default path
    fn new() -> Self {
        let db = jsKVDB::from_path();
        db
    }
}

use crate::{BatchDB, BatchWriter};

pub struct WriteBatch {
    keys : Vec<Vec<u8>>,
    values : Vec<Vec<u8>>,
  }

impl BatchWriter for WriteBatch {
fn new() -> Self {
    WriteBatch {
    keys : Vec::new(),
    values : Vec::new(),
    }
}

    fn batch_put(&mut self, key: &[u8], val: &[u8]) {
    self.keys.push(key.to_vec());
    self.values.push(val.to_vec());
    }
}

use jsKVDB as DB;

impl BatchDB for DB {
    type BatchWrite = WriteBatch;

    fn flush(&mut self, batch: Self::BatchWrite) {
        let keys = batch.keys;
        let jskeys: Vec<_> = keys.into_iter().map(|key| js_sys::Uint8Array::from(&key[..])).collect();
        let vals = batch.values;
        let jsvals: Vec<_> = vals.into_iter().map(|val| js_sys::Uint8Array::from(&val[..])).collect();
        self.jsbatch_put(jskeys, jsvals);
    }
}
