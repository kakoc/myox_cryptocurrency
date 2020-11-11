// use rand::rngs::OsRng;
use crate::transaction::Transaction;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{All, Error, Message, PublicKey, Secp256k1, SecretKey, Signature, Verification};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Wallet {
    secp: Secp256k1<All>,
    pub public_key: PublicKey,
    secret_key: SecretKey,
    pub balance: i64,
}

impl Wallet {
    pub fn new(start_balance: i64) -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);

        Wallet {
            secp,
            secret_key,
            public_key,
            balance: start_balance,
        }
    }

    pub fn sign(&self, data: String) -> Result<Signature, Error> {
        let msg = Sha256::digest(&data.as_bytes());
        let msg = Message::from_slice(&msg)?;

        // println!("long: {}", self.secp.sign(&msg, &self.secret_key));

        Ok(self.secp.sign(&msg, &self.secret_key))
    }

    pub fn verify(&self, data: String, sig: [u8; 64]) -> Result<bool, Error> {
        let msg = Sha256::digest(&data.as_bytes());
        let msg = Message::from_slice(&msg)?;
        let sig = Signature::from_compact(&sig)?;

        Ok(self.secp.verify(&msg, &sig, &self.public_key).is_ok())
    }

    pub fn create_transaction(&mut self, amount: u64, to: PublicKey) -> Option<Transaction> {
        if self.balance < amount as i64 {
            None
        } else {
            Some(Transaction::new(self.clone(), to, amount))
        }
    }
}

pub fn verify(data: String, sig: [u8; 64], public_key: PublicKey) -> Result<bool, Error> {
    let secp = Secp256k1::new();

    let msg = Sha256::digest(&data.as_bytes());
    let msg = Message::from_slice(&msg)?;
    let sig = Signature::from_compact(&sig)?;

    Ok(secp.verify(&msg, &sig, &public_key).is_ok())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let w = Wallet::new(111);

        let public_key_as_hex = format!("{:x}", w.public_key);

        assert_eq!(w.balance, 111);
        assert_eq!(public_key_as_hex.len(), 66);
        assert_eq!(public_key_as_hex.trim().len(), 66);
    }

    #[test]
    fn test_valid_sign_data() {
        let w = Wallet::new(111);

        let s = w.sign("foo".to_owned()).unwrap();
        assert!(w.verify("foo".to_owned(), s.serialize_compact()).unwrap());
    }

    #[test]
    fn test_invalid_sign_data() {
        let w = Wallet::new(111);
    }

    #[test]
    fn transaction_not_created() {
        let mut w = Wallet::new(111);
        let to_w = Wallet::new(0);
        let r = w.create_transaction(11111, to_w.public_key);

        if let Some(r) = r {
            assert!(false);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn transaction_created() {
        let mut w = Wallet::new(111);
        let to_w = Wallet::new(0);
        let r = w.create_transaction(1, to_w.public_key);

        if let Some(r) = r {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn transaction_has_valid_props() {
        let mut w = Wallet::new(111);
        let to_w = Wallet::new(0);
        let r = w.create_transaction(1, to_w.public_key).unwrap();

        assert_eq!(r.input.1, 111);
        assert_eq!(r.input.2, w.public_key);
        assert_eq!(*r.output_map.get(&to_w.public_key).unwrap(), 1);
    }
}

// USEFUL

// let message = Message::from_slice(&[0xab; 32]).expect("32 bytes");
// let seckey = SecretKey::from_slice(&self.secret_key.as_bytes())?;
