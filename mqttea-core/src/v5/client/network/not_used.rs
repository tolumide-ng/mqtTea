// use std::time::Duration;

// use async_channel::Receiver;
// use futures::{AsyncReadExt, AsyncWriteExt};
// // use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWriteExt}, net::TcpStream};

// use crate::v5::commons::{error::MQTTError, packet::Packet};

// use super::{
//     handler::{self, AsyncHandler},
//     state::State,
//     ConnectOptions,
// };

// pub struct Network<S, H>
// where
//     H: handler::AsyncHandler,
//     S: AsyncReadExt + AsyncWriteExt + Unpin, // S: AsyncReadExt + AsyncWriteExt
// {
//     state: State,
//     stream: Option<S>,
//     receiver: Option<Receiver<Packet>>,
//     keep_alive: Duration,
//     handler: H,
// }

// impl<S, H> Network<S, H>
// where
//     H: handler::AsyncHandler,
//     S: AsyncReadExt + AsyncWriteExt + Unpin, // S: AsyncReadExt + AsyncWriteExt
// {
//     // pub async fn connect(&mut self, mut stream: S, handler: &mut H) -> Result<(), MQTTError> {
//     pub async fn connect(&mut self, mut stream: S) -> Result<(), MQTTError> {
//         Ok(())
//     }
// }

// mod comp_confirmation {
//     use tokio::net::TcpStream;
//     use tokio_util::compat::TokioAsyncReadCompatExt;

//     use super::*;

//     struct HST {}
//     impl AsyncHandler for HST {
//         async fn handle(&mut self, packet: Packet) {}
//     }

//     async fn test_impl() {
//         let xxx = HST {};
//         // let stream = tokio::net::TcpStream::connect("whatever").await.unwrap();
//         let stream = TcpStream::connect("example.com:80").await.unwrap();
//         // let stream = tokio::net::TcpStream::connect("hostname").await.unwrap();
//         // let stream = tokio::io::BufStream::new(stream);

//         let stream = stream.compat();

//         let mut xx = Network {
//             state: State::new(ConnectOptions::default()),
//             stream: Some(stream),
//             receiver: None,
//             keep_alive: Duration::new(60, 0),
//             handler: xxx,
//         };

//         let stream = TcpStream::connect("example.com:80").await.unwrap();
//         let stream = stream.compat();
//         let abc = xx.connect(stream);
//     }
// }
