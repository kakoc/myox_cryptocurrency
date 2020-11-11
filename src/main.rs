mod block;
mod blockchain;
mod hashing;
mod pub_sub;
mod transaction;
mod utils;
mod wallet;

use actix_web::{get, post, web, App, HttpServer, Responder};
use block::Block;
use blockchain::Blockchain;
use futures::channel::mpsc::{channel, Receiver, Sender};
use listenfd::ListenFd;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use pub_sub::PubSub;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::sync::Mutex;

// #[get("/{id}/{name}/index.html")]
// async fn index(info: web::Path<(u32, String)>) -> impl Responder {
//     format!("Hello {}! id:{}", info.1, info.0)
// }

struct AppState {
    chain_updater: Sender<String>,
}

#[get("/api/blocks")]
async fn index() -> impl Responder {
    let blockchain = Blockchain::new(vec![Block::get_first_block()]);

    format!("{:?}", blockchain.chain)
}

#[derive(Serialize, Deserialize)]
struct MyInfo {
    a: String,
}

#[post("/api/block")]
async fn add_block(
    state: web::Data<Mutex<AppState>>,
    json: web::Json<MyInfo>,
    // ) -> web::Json<MyInfo> {
) -> impl Responder {
    // let mut chain = state.lock().unwrap();
    // chain.blockchain.add_block(json.a.clone());
    let mut sender = state.lock().unwrap();
    sender
        .chain_updater
        .try_send(json.a.clone())
        .expect("cannot send update through updater");

    // format!("{:?}", chain.blockchain)
    format!("ok")
}

fn run_simulation() {
    let mut blockchain = Blockchain::new(vec![Block::get_first_block()]);
    blockchain.add_block("bla".to_string());
    let mut dfs = vec![];
    for i in 1..1000000 {
        let before = blockchain.chain.last().unwrap().timestamp;
        blockchain.add_block(i.to_string());
        let after = blockchain.chain.last().unwrap().timestamp;
        dfs.push(after - before);
        println!(
            "difficulty: {}, diff: {}, avg: {:.2}",
            blockchain.chain.last().unwrap().difficulty,
            after - before,
            (dfs.iter().sum::<i64>() / dfs.len() as i64) as f64
        );
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // run_simulation();

    let (mut pubsub, s) = PubSub::new(Blockchain::new(vec![Block::get_first_block()]))
        .await
        .unwrap();
    // pubsub.handle_message().await;

    let mut listenfd = ListenFd::from_env();
    let data = web::Data::new(Mutex::new(AppState { chain_updater: s }));
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .service(add_block)
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:3000")?
    };

    futures::join!(server.run(), pubsub.handle_message());

    Ok(())
}
