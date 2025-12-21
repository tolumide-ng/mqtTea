use bytes::{BufMut, Bytes, BytesMut};

pub(crate) trait Write: Sized {
    fn write(&self, buf: &mut BytesMut);
}

impl Write for u8 {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }
}

impl Write for u16 {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(*self);
    }
}

impl Write for u32 {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u32(*self);
    }
}

impl Write for Bytes {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(self.len() as u16);
        buf.extend_from_slice(self);
    }
}

impl Write for String {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(self.len() as u16);
        buf.extend_from_slice(self.as_bytes());
    }
}
