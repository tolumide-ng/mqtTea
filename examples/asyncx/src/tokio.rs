use dotenvy::dotenv;
use mqttea_core::v5::{
    client::{handler::AsyncHandler, network::asyncx::Network, ConnectOptions},
    commons::packet::Packet,
};
use std::env;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub(crate) struct Handler;

impl AsyncHandler for Handler {
    async fn handle(&mut self, packet: Packet) -> () {
        ()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let hostname = env::var("HOSTNAME").expect("HOSTNAME required");
    let username = Some(env::var("USERNAME").expect("USERNAME required"));
    let password = Some(env::var("PASSWORD").expect("PASSWORD required"));

    let mut handler = Handler;

    println!("the hostname is {:?}", hostname);

    let stream = tokio::net::TcpStream::connect(hostname).await.unwrap();

    println!("connected!!!****");
    let stream = tokio::io::BufStream::new(stream);
    let stream = stream.compat();

    let options = ConnectOptions {
        username,
        password,
        ..Default::default()
    };

    let network = Network::new(options, stream).await;
    let (mut result, client) = network.inspect(|x| println!("x {:?}", x)).unwrap();

    tokio::spawn(async move {
        println!("now running!!!!!!!!");
        result.run(&mut handler).await.unwrap();
    });

    println!("ABOUT TO DISCONNECT!!!***");
    client.disconnect().await.unwrap();
    println!("disconnected!!!");
}
