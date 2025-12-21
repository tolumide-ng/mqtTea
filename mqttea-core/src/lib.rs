pub mod constants;
#[cfg(test)]
mod retest_utils;
pub mod v5;

// #[cfg(all(feature = "syncx", feature = "asyncx"))]
// compile_error!("feature \"syncx\" and feature \"asyncx\" cannot be enabled at the same time");

// #[cfg(not(any(feature = "syncx", feature = "asyncx")))]
// compile_error!("Either feature \"syncx\" or feature \"asyncx\" must be enabled");

// pub(crate) trait FromLeBytes {
//     fn from_be_bytes(bytes: &[u8]) -> Self;
// }

// macro_rules! impl_from_be_bytes {
//     ($($ty:ty),*) => {
//         $(
//             impl FromLeBytes for $ty {
//                 fn (bytes: &[u8]) -> Self {
//                     <$ty>::from_be_bytes(bytes.try_into().unwrap())
//                 }
//             }
//         )*
//     };
// }

// // impl_from_be_bytes!(u8, u16, u32);

// macro_rules! validate_packet {
//     (($expr:expr), ($val:expr)) => {
//         if $expr.length() > $val {
//             return Err(MQTTError::MaximumSizeExceed);
//         } else {
//             Ok(())
//         }
//     };
// }
