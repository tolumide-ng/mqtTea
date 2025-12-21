use mqttea_macros::FromU8;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, FromU8)]
pub enum DisconnectReasonCode {
    #[default]
    NormalDisconnection = 0,
    DisconnectWithWillMessage = 4,
    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    ServerBusy = 137,
    ServerShuttingDown = 139,
    KeepAliveTimeout = 141,
    SessionTakenOver = 142,
    TopicFilterInvalid = 143,
    TopicNameInvalid = 144,
    ReceiveMaximumExceeded = 147,
    TopicAliasInvalid = 148,
    PacketTooLarge = 149,
    MessageRateTooHigh = 150,
    QuotaExceeded = 151,
    AdministrativeAction = 152,
    PayloadFormatIndicator = 153,
    RetainNotSupported = 154,
    QoSNotSupported = 155,
    UseAnotherServer = 156,
    ServerMoved = 157,
    SharedSubscriptionsNotSupported = 158,
    ConnectionRateExceeded = 159,
    MaximumConnectTime = 160,
    SubscriptionIdentifiers = 161,
    WildcardSubscriptionsNotSupported = 162,
}