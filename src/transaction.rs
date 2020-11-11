use crate::{
    utils::time_now,
    wallet::{verify, Wallet},
};
use secp256k1::{ffi, PublicKey, Signature};
use std::{collections::HashMap, os::raw::c_uchar};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// type o = HashMap<PublicKey, i64>;
// #[derive(Serialize)]
// struct FfiPublicKey([c_uchar; 64]);
// #[derive(Serialize)]
// struct SerPublicKey(FfiPublicKey);

// struct output(HashMap<PublicKey, i64>);

#[derive(Clone)]
pub struct Transaction {
    id: String,
    from: Wallet,
    to: PublicKey,
    amount: u64,
    pub output_map: HashMap<PublicKey, i64>,
    pub input: (i64, i64, PublicKey, Signature),
}

impl Transaction {
    pub fn new(from: Wallet, to: PublicKey, amount: u64) -> Self {
        if from.balance < amount as i64 {
            panic!("not enough balance");
        }

        let uuid = Uuid::new_v4();

        let mut output_map: HashMap<PublicKey, i64> = HashMap::new();
        output_map.insert(to, amount as i64);
        output_map.insert(from.public_key, from.balance - amount as i64);

        let input = (
            time_now(),
            from.balance,
            from.public_key,
            from.sign(
                serde_json::to_string(&format!("{:?}", output_map)).expect("output map serialized"),
            )
            .expect("transaction signature"),
        );

        Self {
            id: format!("{}", uuid),
            from,
            to,
            amount,
            output_map,
            input,
        }
    }

    pub fn is_valid_transaction(
        Transaction {
            input: (_, initial_balance, public_key, signature),
            output_map,
            ..
        }: Transaction,
    ) -> bool {
        let output_total = output_map.values().sum::<i64>();

        if initial_balance != output_total {
            return false;
        }

        if let Ok(res) = verify(
            serde_json::to_string(&format!("{:?}", output_map)).expect("output map serialized"),
            signature.serialize_compact(),
            public_key,
        ) {
            return res;
        } else {
            return false;
        }
    }

    pub fn update(&mut self, from: Wallet, to: PublicKey, amount: u64) {
        if from.balance < amount as i64 {
            panic!("not enough balance");
        }

        *self.output_map.entry(to).or_insert(0) += amount as i64;
        *self.output_map.get_mut(&from.public_key).unwrap() -= amount as i64;
        self.input = (
            time_now(),
            from.balance,
            from.public_key,
            from.sign(
                serde_json::to_string(&format!("{:?}", self.output_map))
                    .expect("output map serialized"),
            )
            .expect("transaction signature"),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_id() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        assert!(tx.id.len() > 0);
    }

    #[test]
    fn test_state_after_transaction() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        assert_eq!(*tx.output_map.get(&f_wallet.public_key).unwrap(), 111 - 11);
        assert_eq!(*tx.output_map.get(&to_wallet_key).unwrap(), 11);
    }

    #[test]
    fn test_transaction_valid() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        assert_eq!(Transaction::is_valid_transaction(tx), true);
    }

    #[test]
    fn test_transaction_invalid() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let mut tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        *tx.output_map.get_mut(&f_wallet.public_key).unwrap() = 1;
        assert_eq!(Transaction::is_valid_transaction(tx.clone()), false);
        *tx.output_map.get_mut(&f_wallet.public_key).unwrap() = 111 - 11;
        assert_eq!(Transaction::is_valid_transaction(tx.clone()), true);
        tx.input.3 = Wallet::new(228).sign("foo".to_owned()).unwrap();
        assert_eq!(Transaction::is_valid_transaction(tx), false);
    }

    #[test]
    fn test_transaction_next_transaction() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let mut tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        assert_eq!(Transaction::is_valid_transaction(tx.clone()), true);
        let fst_signature = tx.input.3.clone();

        let to_wallet_key2 = Wallet::new(5).public_key;
        tx.update(f_wallet.clone(), to_wallet_key2, 5);

        assert_eq!(*tx.output_map.get(&f_wallet.public_key).unwrap(), 95);
        assert_eq!(*tx.output_map.get(&to_wallet_key).unwrap(), 11);
        assert_eq!(*tx.output_map.get(&to_wallet_key2).unwrap(), 5);
        assert_eq!(tx.input.1, tx.output_map.values().sum::<i64>());
        assert_ne!(tx.input.3, fst_signature);
    }

    #[test]
    fn test_transaction_next_transaction_with_same_recipient() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let mut tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        assert_eq!(Transaction::is_valid_transaction(tx.clone()), true);
        tx.update(f_wallet.clone(), to_wallet_key, 5);

        assert_eq!(*tx.output_map.get(&f_wallet.public_key).unwrap(), 95);
        assert_eq!(*tx.output_map.get(&to_wallet_key).unwrap(), 16);
        assert_eq!(tx.input.1, tx.output_map.values().sum::<i64>());
    }

    #[test]
    #[should_panic]
    fn test_transaction_from_lower_than_amount() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        Transaction::new(f_wallet.clone(), to_wallet_key, 11111);
    }

    #[test]
    #[should_panic]
    fn test_transaction_update_high_amount() {
        let f_wallet = Wallet::new(111);
        let to_wallet_key = Wallet::new(111).public_key;

        let mut tx = Transaction::new(f_wallet.clone(), to_wallet_key, 11);
        let to_wallet_key2 = Wallet::new(5).public_key;
        tx.update(f_wallet.clone(), to_wallet_key2, 11111);
    }
}
