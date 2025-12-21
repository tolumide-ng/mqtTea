use std::borrow::Cow;
use std::fmt::Display;
use std::future::Future;

use bytes::{Bytes, BytesMut};
use futures::AsyncWriteExt;

use crate::v5::commons::error::MQTTError;
use crate::v5::traits::read_data::ReadData;
use crate::v5::traits::utils::Utils;
use crate::v5::traits::{self, syncx::read::Read};

/// Must be encoded using the VBI
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Property<'a> {
    PayloadFormatIndicator(Option<u8>) = 1,
    MessageExpiryInterval(Option<u32>) = 2,
    ContentType(Option<Cow<'a, str>>) = 3,
    ResponseTopic(Option<Cow<'a, str>>) = 8,
    CorrelationData(Option<Cow<'a, [u8]>>) = 9,
    SubscriptionIdentifier(Cow<'a, usize>) = 11,
    SessionExpiryInterval(Option<u32>) = 17,
    AssignedClientIdentifier(Option<Cow<'a, str>>) = 18,
    ServerKeepAlive(Option<u16>) = 19,
    AuthenticationMethod(Option<Cow<'a, str>>) = 21,
    AuthenticationData(Option<Cow<'a, [u8]>>) = 22,
    RequestProblemInformation(Option<u8>) = 23,
    WillDelayInterval(Option<u32>) = 24,
    RequestResponseInformation(Option<u8>) = 25,
    ResponseInformation(Option<Cow<'a, str>>) = 26,
    ServerReference(Option<Cow<'a, str>>) = 28,
    ReasonString(Option<Cow<'a, str>>) = 31,
    ReceiveMaximum(Option<u16>) = 33,
    TopicAliasMaximum(Option<u16>) = 34,
    TopicAlias(Option<u16>) = 35,
    MaximumQoS(Option<u8>) = 36,
    RetainAvailable(Option<u8>) = 37,
    UserProperty(Cow<'a, (String, String)>) = 38,
    MaximumPacketSize(Option<u32>) = 39,
    WildCardSubscription(Option<u8>) = 40,
    SubscriptionIdentifierAvailable(Option<u8>) = 41,
    SharedSubscriptionAvailable(Option<u8>) = 42,
}

impl<'a> ReadData for Property<'a> {}

impl<'a> From<&Property<'a>> for u8 {
    fn from(value: &Property) -> Self {
        match value {
            Property::PayloadFormatIndicator(_) => 1,
            Property::MessageExpiryInterval(_) => 2,
            Property::ContentType(_) => 3,
            Property::ResponseTopic(_) => 8,
            Property::CorrelationData(_) => 9,
            Property::SubscriptionIdentifier(_) => 11,
            Property::SessionExpiryInterval(_) => 17,
            Property::AssignedClientIdentifier(_) => 18,
            Property::ServerKeepAlive(_) => 19,
            Property::AuthenticationMethod(_) => 21,
            Property::AuthenticationData(_) => 22,
            Property::RequestProblemInformation(_) => 23,
            Property::WillDelayInterval(_) => 24,
            Property::RequestResponseInformation(_) => 25,
            Property::ResponseInformation(_) => 26,
            Property::ServerReference(_) => 28,
            Property::ReasonString(_) => 31,
            Property::ReceiveMaximum(_) => 33,
            Property::TopicAliasMaximum(_) => 34,
            Property::TopicAlias(_) => 35,
            Property::MaximumQoS(_) => 36,
            Property::RetainAvailable(_) => 37,
            Property::UserProperty(_) => 38,
            Property::MaximumPacketSize(_) => 39,
            Property::WildCardSubscription(_) => 40,
            Property::SubscriptionIdentifierAvailable(_) => 41,
            Property::SharedSubscriptionAvailable(_) => 42,
        }
    }
}

impl<'a> TryFrom<u8> for Property<'a> {
    type Error = MQTTError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Property::PayloadFormatIndicator(None)),
            2 => Ok(Property::MessageExpiryInterval(None)),
            3 => Ok(Property::ContentType(None)),
            8 => Ok(Property::ResponseTopic(None)),
            9 => Ok(Property::CorrelationData(None)),
            11 => Ok(Property::SubscriptionIdentifier(Cow::Owned(0))),
            17 => Ok(Property::SessionExpiryInterval(None)),
            18 => Ok(Property::AssignedClientIdentifier(None)),
            19 => Ok(Property::ServerKeepAlive(None)),
            21 => Ok(Property::AuthenticationMethod(None)),
            22 => Ok(Property::AuthenticationData(None)),
            23 => Ok(Property::RequestProblemInformation(None)),
            24 => Ok(Property::WillDelayInterval(None)),
            25 => Ok(Property::RequestResponseInformation(None)),
            26 => Ok(Property::ResponseInformation(None)),
            28 => Ok(Property::ServerReference(None)),
            31 => Ok(Property::ReasonString(None)),
            33 => Ok(Property::ReceiveMaximum(None)),
            34 => Ok(Property::TopicAliasMaximum(None)),
            35 => Ok(Property::TopicAlias(None)),
            36 => Ok(Property::MaximumQoS(None)),
            37 => Ok(Property::RetainAvailable(None)),
            38 => Ok(Property::UserProperty(Cow::Owned((
                String::from(""),
                String::from(""),
            )))),
            39 => Ok(Property::MaximumPacketSize(None)),
            40 => Ok(Property::WildCardSubscription(None)),
            41 => Ok(Property::SubscriptionIdentifierAvailable(None)),
            42 => Ok(Property::SharedSubscriptionAvailable(None)),
            v => Err(MQTTError::UnknownProperty(v)),
        }
    }
}

impl<'a> Property<'a> {
    fn with_id<F>(&self, buf: &mut BytesMut, func: F)
    where
        F: Fn(&mut BytesMut),
    {
        use traits::syncx::write::Write;
        u8::from(self).write(buf);
        // buf.put_u8(u8::from(self));
        func(buf);
    }

    async fn write_to_stream<S, T>(&self, stream: &mut S, value: &T) -> Result<(), MQTTError>
    where
        S: AsyncWriteExt + Unpin,
        T: traits::asyncx::write::Write<S>,
    {
        use traits::asyncx::write::Write;
        u8::from(self).write(stream).await?;
        value.write(stream).await
    }

    async fn write_async<'b, S, F, Fut>(&self, stream: &'b mut S, func: F) -> Result<(), MQTTError>
    where
        S: AsyncWriteExt + Unpin + 'b,
        F: FnOnce(&'b mut S) -> Fut,
        Fut: Future<Output = Result<(), MQTTError>> + 'b,
    {
        use traits::asyncx::write::Write;
        u8::from(self).write(stream).await?;

        func(stream).await
    }

    pub(crate) fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        if buf.is_empty() {
            return Err(MQTTError::IncompleteData("MQTT Property", 1, 0));
        }

        match u8::read(buf).unwrap() {
            1 => Ok(Property::PayloadFormatIndicator(Some(u8::read(buf)?))),
            2 => Ok(Property::MessageExpiryInterval(Some(u32::read(buf)?))),
            3 => Ok(Property::ContentType(Some(Cow::Owned(String::read(buf)?)))),
            8 => Ok(Property::ResponseTopic(Some(Cow::Owned(String::read(
                buf,
            )?)))),
            9 => Ok(Property::CorrelationData(Some(Cow::Owned(
                Bytes::read(buf)?.to_vec(),
            )))),
            11 => Ok(Property::SubscriptionIdentifier(Cow::Owned(
                Self::decode(buf)?.0,
            ))),
            17 => Ok(Property::SessionExpiryInterval(Some(u32::read(buf)?))),
            18 => Ok(Property::AssignedClientIdentifier(Some(Cow::Owned(
                String::read(buf)?,
            )))),
            19 => Ok(Property::ServerKeepAlive(Some(u16::read(buf)?))),
            21 => Ok(Property::AuthenticationMethod(Some(Cow::Owned(
                String::read(buf)?,
            )))),
            22 => Ok(Property::AuthenticationData(Some(Cow::Owned(
                (Bytes::read(buf)?).to_vec(),
            )))),
            23 => Ok(Property::RequestProblemInformation(Some(u8::read(buf)?))),
            24 => Ok(Property::WillDelayInterval(Some(u32::read(buf)?))),
            25 => Ok(Property::RequestResponseInformation(Some(u8::read(buf)?))),
            26 => Ok(Property::ResponseInformation(Some(Cow::Owned(
                String::read(buf)?,
            )))),
            28 => Ok(Property::ServerReference(Some(Cow::Owned(String::read(
                buf,
            )?)))),
            31 => Ok(Property::ReasonString(Some(Cow::Owned(String::read(buf)?)))),
            33 => Ok(Property::ReceiveMaximum(Some(u16::read(buf)?))),
            34 => Ok(Property::TopicAliasMaximum(Some(u16::read(buf)?))),
            35 => Ok(Property::TopicAlias(Some(u16::read(buf)?))),
            36 => Ok(Property::MaximumQoS(Some(u8::read(buf)?))),
            37 => Ok(Property::RetainAvailable(Some(u8::read(buf)?))),
            38 => Ok(Property::UserProperty(Cow::Owned((
                String::read(buf)?,
                String::read(buf)?,
            )))),
            39 => Ok(Property::MaximumPacketSize(Some(u32::read(buf)?))),
            40 => Ok(Property::WildCardSubscription(Some(u8::read(buf)?))),
            41 => Ok(Property::SubscriptionIdentifierAvailable(Some(u8::read(
                buf,
            )?))),
            42 => Ok(Property::SharedSubscriptionAvailable(Some(u8::read(buf)?))),
            v => Err(MQTTError::UnknownProperty(v)),
        }
    }
}

/// this would eventually be changed to use derive_more lib
impl<'a> Display for Property<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "this would be changed eventually to use derive_more::Error"
        )
    }
}

pub(crate) mod synx {
    use bytes::{Bytes, BytesMut};
    use std::borrow::Cow;

    use crate::v5::commons::error::MQTTError;
    use crate::v5::traits::bufferio::BufferIO;
    use crate::v5::traits::{syncx::read::Read, syncx::write::Write};

    use super::Property;

    // #[cfg(feature = "sync")]
    impl<'a> BufferIO for Property<'a> {
        fn length(&self) -> usize {
            match self {
                Self::SubscriptionIdentifier(length) => **length,
                _ => 0,
            }
        }

        fn write(&self, buf: &mut BytesMut) -> Result<(), MQTTError> {
            match self {
                Self::SessionExpiryInterval(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::ReceiveMaximum(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::MaximumPacketSize(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::TopicAliasMaximum(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::TopicAlias(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::RequestResponseInformation(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::RequestProblemInformation(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::UserProperty(Cow::Borrowed((ref k, ref v))) => self.with_id(buf, |b| {
                    k.write(b);
                    v.write(b);
                }),
                Self::AuthenticationMethod(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::AuthenticationData(Some(p)) => {
                    self.with_id(buf, |b| Bytes::from_iter(p.to_vec()).write(b))
                }
                Self::WillDelayInterval(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::PayloadFormatIndicator(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::MessageExpiryInterval(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::ContentType(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::ResponseTopic(Some(p)) => {
                    self.with_id(buf, |b| Bytes::from_iter(p.as_bytes().to_vec()).write(b))
                }
                Self::CorrelationData(Some(p)) => {
                    self.with_id(buf, |b| Bytes::from_iter(p.to_vec()).write(b))
                }
                // NOTE: this needs to be tested for if this method of writing is correct or not!
                Self::SubscriptionIdentifier(_) => self.with_id(buf, |b| self.encode(b).unwrap()),
                Self::AssignedClientIdentifier(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::ServerKeepAlive(Some(p)) => self.with_id(buf, |b| p.write(b)),
                Self::ResponseInformation(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::ServerReference(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::ReasonString(Some(data)) => {
                    self.with_id(buf, |b| Bytes::from_iter(data.as_bytes().to_vec()).write(b))
                }
                Self::MaximumQoS(Some(i)) => self.with_id(buf, |b| i.write(b)),
                Self::RetainAvailable(Some(i)) => self.with_id(buf, |b| i.write(b)),
                Self::WildCardSubscription(Some(i)) => self.with_id(buf, |b| i.write(b)),
                Self::SubscriptionIdentifierAvailable(Some(i)) => self.with_id(buf, |b| i.write(b)),
                Self::SharedSubscriptionAvailable(Some(i)) => self.with_id(buf, |b| i.write(b)),
                // _ => {unreachable!("Unrecognized enum variant or argument!")}
                _ => {}
            }

            Ok(())
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            if buf.is_empty() {
                return Err(MQTTError::IncompleteData("MQTT Property", 1, 0));
            }

            match u8::read(buf).unwrap() {
                1 => Ok(Property::PayloadFormatIndicator(Some(u8::read(buf)?))),
                2 => Ok(Property::MessageExpiryInterval(Some(u32::read(buf)?))),
                3 => Ok(Property::ContentType(Some(Cow::Owned(String::read(buf)?)))),
                8 => Ok(Property::ResponseTopic(Some(Cow::Owned(String::read(
                    buf,
                )?)))),
                9 => Ok(Property::CorrelationData(Some(Cow::Owned(
                    Bytes::read(buf)?.to_vec(),
                )))),
                11 => Ok(Property::SubscriptionIdentifier(Cow::Owned(
                    Self::decode(buf)?.0,
                ))),
                17 => Ok(Property::SessionExpiryInterval(Some(u32::read(buf)?))),
                18 => Ok(Property::AssignedClientIdentifier(Some(Cow::Owned(
                    String::read(buf)?,
                )))),
                19 => Ok(Property::ServerKeepAlive(Some(u16::read(buf)?))),
                21 => Ok(Property::AuthenticationMethod(Some(Cow::Owned(
                    String::read(buf)?,
                )))),
                22 => Ok(Property::AuthenticationData(Some(Cow::Owned(
                    (Bytes::read(buf)?).to_vec(),
                )))),
                23 => Ok(Property::RequestProblemInformation(Some(u8::read(buf)?))),
                24 => Ok(Property::WillDelayInterval(Some(u32::read(buf)?))),
                25 => Ok(Property::RequestResponseInformation(Some(u8::read(buf)?))),
                26 => Ok(Property::ResponseInformation(Some(Cow::Owned(
                    String::read(buf)?,
                )))),
                28 => Ok(Property::ServerReference(Some(Cow::Owned(String::read(
                    buf,
                )?)))),
                31 => Ok(Property::ReasonString(Some(Cow::Owned(String::read(buf)?)))),
                33 => Ok(Property::ReceiveMaximum(Some(u16::read(buf)?))),
                34 => Ok(Property::TopicAliasMaximum(Some(u16::read(buf)?))),
                35 => Ok(Property::TopicAlias(Some(u16::read(buf)?))),
                36 => Ok(Property::MaximumQoS(Some(u8::read(buf)?))),
                37 => Ok(Property::RetainAvailable(Some(u8::read(buf)?))),
                38 => Ok(Property::UserProperty(Cow::Owned((
                    String::read(buf)?,
                    String::read(buf)?,
                )))),
                39 => Ok(Property::MaximumPacketSize(Some(u32::read(buf)?))),
                40 => Ok(Property::WildCardSubscription(Some(u8::read(buf)?))),
                41 => Ok(Property::SubscriptionIdentifierAvailable(Some(u8::read(
                    buf,
                )?))),
                42 => Ok(Property::SharedSubscriptionAvailable(Some(u8::read(buf)?))),
                v => Err(MQTTError::UnknownProperty(v)),
            }
        }
    }
}

pub(crate) mod asyncx {
    use std::borrow::Cow;

    use crate::v5::commons::error::MQTTError;
    use crate::v5::traits::asyncx::read::Read;
    use crate::v5::traits::streamio::StreamIO;

    use super::Property;

    impl<'a> StreamIO for Property<'a> {
        fn length(&self) -> usize {
            match self {
                Self::SubscriptionIdentifier(length) => **length,
                _ => 0,
            }
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
        {
            let property_id = u8::read(stream).await?;

            match property_id {
                1 => Ok(Property::PayloadFormatIndicator(Some(
                    u8::read(stream).await?,
                ))),
                2 => Ok(Property::MessageExpiryInterval(Some(
                    u32::read(stream).await?,
                ))),
                3 => Ok(Property::ContentType(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                8 => Ok(Property::CorrelationData(Some(Cow::Owned(
                    Vec::read(stream).await?,
                )))),
                11 => Ok(Property::SubscriptionIdentifier(Cow::Owned(
                    Self::decode(stream).await?.0,
                ))),
                17 => Ok(Property::SessionExpiryInterval(Some(
                    u32::read(stream).await?,
                ))),
                18 => Ok(Property::AssignedClientIdentifier(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                19 => Ok(Property::ServerKeepAlive(Some(u16::read(stream).await?))),
                21 => Ok(Property::AuthenticationMethod(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                22 => Ok(Property::AuthenticationData(Some(Cow::Owned(
                    Vec::read(stream).await?,
                )))),
                23 => Ok(Property::RequestProblemInformation(Some(
                    u8::read(stream).await?,
                ))),
                24 => Ok(Property::WillDelayInterval(Some(u32::read(stream).await?))),
                25 => Ok(Property::RequestResponseInformation(Some(
                    u8::read(stream).await?,
                ))),
                26 => Ok(Property::ResponseInformation(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                28 => Ok(Property::ServerReference(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                31 => Ok(Property::ReasonString(Some(Cow::Owned(
                    String::read(stream).await?,
                )))),
                33 => Ok(Property::ReceiveMaximum(Some(u16::read(stream).await?))),
                34 => Ok(Property::TopicAliasMaximum(Some(u16::read(stream).await?))),
                35 => Ok(Property::TopicAlias(Some(u16::read(stream).await?))),
                36 => Ok(Property::MaximumQoS(Some(u8::read(stream).await?))),
                37 => Ok(Property::RetainAvailable(Some(u8::read(stream).await?))),
                38 => Ok(Property::UserProperty(Cow::Owned((
                    String::read(stream).await?,
                    String::read(stream).await?,
                )))),
                39 => Ok(Property::MaximumPacketSize(Some(u32::read(stream).await?))),
                40 => Ok(Property::WildCardSubscription(Some(
                    u8::read(stream).await?,
                ))),
                41 => Ok(Property::SubscriptionIdentifierAvailable(Some(
                    u8::read(stream).await?,
                ))),
                42 => Ok(Property::SharedSubscriptionAvailable(Some(
                    u8::read(stream).await?,
                ))),
                v => Err(MQTTError::UnknownProperty(v)),
            }
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            match self {
                Self::SessionExpiryInterval(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::ReceiveMaximum(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::MaximumPacketSize(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::TopicAliasMaximum(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::TopicAlias(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::RequestResponseInformation(Some(p)) => {
                    self.write_to_stream(stream, p).await?
                }
                Self::RequestProblemInformation(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::UserProperty(Cow::Borrowed(p)) => self.write_to_stream(stream, *p).await?,
                Self::AuthenticationMethod(Some(p)) => {
                    self.write_to_stream(stream, &p.to_string()).await?
                }
                Self::AuthenticationData(Some(p)) => {
                    self.write_to_stream(stream, &p.to_vec()).await?
                }
                Self::WillDelayInterval(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::PayloadFormatIndicator(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::MessageExpiryInterval(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::ContentType(Some(p)) => self.write_to_stream(stream, &p.to_string()).await?,
                Self::ResponseTopic(Some(p)) => {
                    self.write_to_stream(stream, &p.to_string()).await?
                }
                Self::CorrelationData(Some(p)) => self.write_to_stream(stream, &p.to_vec()).await?,
                Self::SubscriptionIdentifier(_) => {
                    self.write_async(stream, |s| async move { self.encode(s).await })
                        .await?;
                }
                Self::AssignedClientIdentifier(Some(p)) => {
                    self.write_to_stream(stream, &p.to_string()).await?
                }
                Self::ServerKeepAlive(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::ResponseInformation(Some(p)) => {
                    self.write_to_stream(stream, &p.to_string()).await?
                }
                Self::ServerReference(Some(p)) => {
                    self.write_to_stream(stream, &p.to_string()).await?
                }
                Self::ReasonString(Some(p)) => self.write_to_stream(stream, &p.to_string()).await?,
                Self::MaximumQoS(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::RetainAvailable(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::WildCardSubscription(Some(p)) => self.write_to_stream(stream, p).await?,
                Self::SubscriptionIdentifierAvailable(Some(p)) => {
                    self.write_to_stream(stream, p).await?
                }
                Self::SharedSubscriptionAvailable(Some(p)) => {
                    self.write_to_stream(stream, p).await?
                }
                _ => (),
            }

            Ok(())
        }
    }
}
