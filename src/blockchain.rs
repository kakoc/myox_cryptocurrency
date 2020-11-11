use crate::block::Block;
use crate::hashing::gen_hash;

use chrono::Utc;
use proptest::prelude::*;
use rand::Rng;

const MINING_SPEED: usize = 1000;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { chain: blocks }
    }

    pub fn add_block(&mut self, data: String) {
        self.chain
            .push(Block::new_with_previous(data, self.chain.last().unwrap()))
    }

    pub fn get_nth_block(&self, i: usize) -> Option<&Block> {
        self.chain.get(i)
    }

    pub fn is_valid_chain(chain: &Blockchain) -> bool {
        if *chain.get_nth_block(0).unwrap() != Block::get_first_block() {
            return false;
        }

        for i in 1..chain.chain.len() {
            if chain.chain[i - 1].hash != chain.chain[i].prev_hash {
                return false;
            };

            let hash = gen_hash(vec![
                chain.chain[i].timestamp.to_string(),
                chain.chain[i].data.to_string(),
                chain.chain[i - 1].hash.to_string(),
                chain.chain[i].difficulty.to_string(),
                chain.chain[i].nonce.to_string(),
            ]);

            if hash != chain.chain[i].hash {
                return false;
            }
        }

        true
    }

    pub fn replace_chain(&mut self, chain: &Blockchain) {
        if chain.chain.len() > self.chain.len() && Blockchain::is_valid_chain(chain) {
            self.chain = chain.chain.clone();
        }
    }

    pub fn optimize_difficulty(block: &Block, timestamp: i64) -> usize {
        // return 2;
        let d = block.difficulty;
        let t = block.timestamp;

        if d < 1 {
            return 1;
        }

        if timestamp as usize > MINING_SPEED {
            d - 1
        } else {
            d + 1
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn gen_data_vec(n: usize) -> impl Strategy<Value = Vec<String>> {
        proptest::collection::vec(".*", 3..n)
    }

    proptest! {
        #[test]
        fn test_chain_validity(data in gen_data_vec(5)) {
        // Utc::now().timestamp().to_string(),
            let mut chain = vec![Block::get_first_block()];
            let mut blockchain = Blockchain::new(vec![Block::get_first_block()]);

            for (i,v) in data.iter().enumerate() {
                chain.push(Block::new_with_previous(v.clone(), chain.get(i).unwrap()));
                blockchain.add_block(v.to_string());
            }

            for i in 1..chain.len() {
                let hash = gen_hash(vec![
                    blockchain.chain[i].timestamp.to_string(),
                    blockchain.chain[i].data.to_string(),
                    blockchain.chain[i-1].hash.to_string(),
                    blockchain.chain[i].difficulty.to_string(),
                    blockchain.chain[i].nonce.to_string(),
                ]);
                chain[i].hash = hash;
            }

            assert!(Blockchain::is_valid_chain(&blockchain));
            assert_eq!(blockchain.chain.len(), data.len() + 1);
            assert_eq!(*blockchain.get_nth_block(0).unwrap() , Block::get_first_block());
            for i in 0..chain.len() {
                assert_eq!(chain[i].hash, *blockchain.get_nth_block(i).unwrap().hash);
            }
        }

        #[test]
        fn test_broken_chain(data in gen_data_vec(5)) {
            let mut blockchain = Blockchain::new(vec![Block::get_first_block()]);

            for (i,v) in data.iter().enumerate() {
                blockchain.add_block(v.to_string());
            }

            let chain_len = blockchain.chain.len();
            blockchain.chain.get_mut(rand::thread_rng().gen_range(2, chain_len)).unwrap().data = "bar".to_string();

            assert_eq!(Blockchain::is_valid_chain(&blockchain), false);
            assert_eq!(blockchain.chain.len(), data.len() + 1);
        }

        #[test]
        fn test_replace_chain(data in gen_data_vec(5), data2 in gen_data_vec(5)) {
            let mut blockchain = Blockchain::new(vec![Block::get_first_block()]);
            let mut blockchain2 = Blockchain::new(vec![Block::get_first_block()]);

            for (i,v) in data.iter().enumerate() {
                blockchain.add_block(v.to_string());
            }

            for (i,v) in data2.iter().enumerate() {
                blockchain2.add_block(v.to_string());
            }

            blockchain.replace_chain(&blockchain2);

            assert_eq!(blockchain.chain.len(), std::cmp::max(data.len(), data2.len()) + 1);
        }
    }
}
