use crate::{
    blockchain::Blockchain,
    hashing::{gen_hash, to_binary},
    utils::time_now,
};

use chrono::Utc;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Block {
    pub timestamp: i64,
    pub data: String,
    pub hash: String,
    pub prev_hash: String,
    pub difficulty: usize,
    pub nonce: usize,
}

impl Block {
    pub fn new_with_previous<T>(data: T, prev_block: &Block) -> Self
    where
        T: AsRef<str>,
    {
        let mut date_now = time_now();
        let mut nonce = 0;
        let mut difficulty =
            Blockchain::optimize_difficulty(&prev_block, date_now - prev_block.timestamp);
        let mut hash = gen_hash(vec![
            date_now.to_string(),
            (*data.as_ref()).to_string(),
            prev_block.hash.to_string(),
            difficulty.to_string(),
            nonce.to_string(),
        ]);

        while hash[0..difficulty] != "0".repeat(difficulty) {
            nonce += 1;
            date_now = time_now();
            difficulty =
                Blockchain::optimize_difficulty(&prev_block, date_now - prev_block.timestamp);
            // println!("{}", difficulty);
            // println!("{}", hash);
            // println!("{}", hash);
            hash = gen_hash(vec![
                date_now.to_string(),
                (*data.as_ref()).to_string(),
                prev_block.hash.to_string(),
                difficulty.to_string(),
                nonce.to_string(),
            ]);
        }

        Self {
            timestamp: date_now,
            data: (*data.as_ref()).to_string(),
            hash,
            prev_hash: prev_block.hash.to_string(),
            difficulty,
            nonce,
        }
    }

    pub fn get_first_block() -> Self {
        Self {
            timestamp: 0,
            data: "empty_data".to_string(),
            hash: "empty_hash".to_string(),
            prev_hash: "empty_previous_hash".to_string(),
            difficulty: 3,
            nonce: 0,
        }
    }
}

proptest! {
    #[test]
    fn test_block_creation(data in ".*") {
        let block = Block::new_with_previous(data.clone(), &Block::get_first_block());
        let hash = gen_hash(vec![
            block.timestamp.to_string(),
            data.to_owned(),
            Block::get_first_block().hash.to_owned(),
            block.difficulty.to_string(),
            block.nonce.to_string(),
        ]);
        assert_eq!(block.data, data);
        assert_eq!(block.hash, hash);
        assert_eq!(block.prev_hash, Block::get_first_block().hash.to_owned());
    }
}
