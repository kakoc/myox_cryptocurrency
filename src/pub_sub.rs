use crate::{block::Block, blockchain::Blockchain};
use futures_util::StreamExt as _;
use redis::AsyncCommands;
use redis::{Commands, Connection, RedisResult, Value};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    pin_mut,
};
use serde_json::json;

const CHANNELS: &'static [&'static str] = &["TEST", "BLOCKCHAIN"];

pub struct PubSub {
    pub_sub_c: redis::aio::PubSub,
    publish_c: redis::aio::Connection,
    blockchain: Blockchain,
    receiver: Receiver<String>,
}

impl PubSub {
    pub async fn new(blockchain: Blockchain) -> redis::RedisResult<(Self, Sender<String>)> {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let publish_conn = client.get_async_connection().await?;
        let pubsub_conn = client.get_async_connection().await?.into_pubsub();
        let (s, r) = channel(1024);

        Ok((
            Self {
                pub_sub_c: pubsub_conn,
                publish_c: publish_conn,
                blockchain,
                receiver: r,
            },
            s,
        ))
    }

    pub async fn handle_message(&mut self) {
        self.pub_sub_c.subscribe("BLOCKCHAIN").await.unwrap();
        let mut pubsub_stream = self.pub_sub_c.on_message().fuse();

        loop {
            futures::select! {
                msg = pubsub_stream.next() => {
                    let chains: Blockchain = msg.expect("valid json").get_payload().unwrap();

                    println!("created blockchain: {:?}", chains);
                },
                data = self.receiver.next() => {
                    if let Some(s) = data {

                    self.blockchain.add_block(s);
                    println!("new chain added");
                    println!("{:?}", self.blockchain);
                    let s = json!(self.blockchain.chain).to_string();
                    self.publish_c
                        .publish::<String, String, usize>("BLOCKCHAIN".to_owned(), s).await.unwrap();
                    }
               }
            }
        }
    }
}

impl redis::FromRedisValue for Blockchain {
    fn from_redis_value(v: &Value) -> RedisResult<Blockchain> {
        match v {
            Value::Data(msg) => {
                let vv: Vec<Block> =
                    serde_json::from_str(std::str::from_utf8(msg).unwrap()).unwrap();
                Ok(Blockchain::new(vv))
            }
            _ => todo!(),
        }
    }
}

// USEFUL

// while let Some(msg) = pubsub_stream.next().await {
//     let chains: Blockchain = msg.get_payload().unwrap();
//     // self.blockchain.replace_chain(msg.get_payload().unwrap());
//     // println!("got message: {:?}", msg);
//     println!("created blockchain: {:?}", chains);
// }
