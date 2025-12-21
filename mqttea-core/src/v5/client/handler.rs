use crate::v5::commons::packet::Packet;

pub trait AsyncHandler {
    fn handle(&mut self, packet: Packet) -> impl std::future::Future<Output = ()> + Send + Sync;
}
