use std::{future::Future, pin::Pin};

use futures::future;

use crate::v5::commons::error::MQTTError;

use super::{packet_id::PacketIdManager, ConnectOptions};

#[cfg(feature = "asyncx")]
pub mod asyncx;
#[cfg(feature = "syncx")]
pub mod syncx;

// mod not_used;

// pub(crate) trait Network: Send + Unpin + Sync {
//     type Output: Send + Unpin + Sync;

//     type LocalResult<'a>: Future<Output = Self::Output> + 'a
//     where
//         Self: 'a;

//     // or rename to setup
//     fn new<'a>(options: ConnectOptions, pkids: PacketIdManager) -> Self;

//     fn connect<'a>(&'a self) -> Self::LocalResult<'a>;
// }

// pub(crate) trait SyncNetwork: Send + Sync + Unpin {
//     type Output;

//     fn new(options: ConnectOptions, pkids: PacketIdManager) -> Self;

//     fn connect<'a>(&self) -> Self::Output;
// }

// #[cfg(feature = "syncx")]
// mod tsyncx {
//     use super::*;
//     // Blanet that can now be globally exposed and called simply as `new` or `connect`
//     impl<T> Network for T
//     where
//         T: SyncNetwork,
//     {
//         type Output = T::Output;

//         type LocalResult<'a>
//             = future::Ready<Self::Output>
//         where
//             Self: 'a;

//         fn new(options: ConnectOptions, pkids: PacketIdManager) -> Self {
//             T::new(options, pkids)
//         }

//         fn connect<'a>(&'a self) -> Self::LocalResult<'a> {
//             futures::future::ready(self.connect())
//         }
//     }
// }

// // #[cfg(feature = "asyncx")]
// // mod tasyncx {
// //     use crate::v5::client::ConnectOptions;

// //     use super::Network;

// //     pub(crate) trait AsyncNetwork: Unpin + Sync + Send {
// //         type Output: Send + Unpin + Sync;

// //         fn new(options: ConnectOptions, pkids: PacketIdManager) -> Self;

// //         async fn connect(&self) -> Self::Output;
// //     }

// //     impl<T> Network for T
// //     where
// //         T: AsyncNetwork,
// //     {
// //         type Output = T::Output;

// //         type LocalResult<'a>
// //             = Pin<Box<dyn Future<Output = Self::Output> + 'a>>
// //         where
// //             Self: 'a;

// //         fn new(options: ConnectOptions, pkids: PacketIdManager) -> Self {
// //             T::new(options, pkids)
// //         }

// //         fn connect<'a>(&'a self) -> Self::LocalResult<'a> {
// //             Box::pin(self.connect())
// //         }
// //     }
// // }
