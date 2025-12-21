use mqttea_macros::FromU8;

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, FromU8, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ConnAckReasonCode {
    #[default]
    Success = 0,
    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError =131,
    UnSupportedProtocolVersion = 132,
    ClientIdentifierNotValid = 133,
    BadUserNameOrPassword = 134,
    NotAuthorized = 135,
    ServerUnAvailable = 136,
    ServerBusy = 137,
    Banned = 138,
    BadAuthenticationMethod = 140,
    TopicNameInvalid = 144,
    PacketTooLarge = 149,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
    RetainNotSupported = 154,
    QoSNotSupported = 155,
    UseAnotherServer = 156,
    ServeMoved = 157,
    ConnectionRateExceeded = 159,
}
