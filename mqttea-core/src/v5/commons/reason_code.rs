#[allow(dead_code)]
#[derive(derive_more::Display)]
#[repr(u8)]
pub(crate) enum ReasonCode {
    /// CONNACK, PUBACK. PUBREC, PUBREL, PUBCOMP, UNSUBACK, AUTH (0x00)
    #[display("Success")]
    Success,  // 0x00
    /// NormalDisconnection = 0x00
    #[display("Normal Disconnection")]
    NormalDisconnection,
    /// SUBACK  = 0x00
    #[display("Granted QoS 0")]
    GrantedQoS0,
    /// SUBACK = 0x01
    #[display("Granted QoS 1")]
    GrantedQoS1,
    /// SUBACK = 0x02
    #[display("Granted QoS 2")]
    GrantedQoS2,
    /// DISCONNECT = 0x04
    #[display("Disconnect with Will Message")]
    DisconnectWithWillMessage,
    /// PUBACK, PUBREC = 0x10
    #[display("No matching subscribers")]
    NoMatchingSubscribers,
    /// UNSUBACK = 0x11
    #[display("No Subscription existed")]
    NoSubscriptionExisted,
    /// AUTH = 0x18
    #[display("Continue authentication")]
    ContinueAuthentication,
    /// AUTH = 0x19
    #[display("Re-authenticate")]
    ReAuthenticate,
    /// CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT = 0x80
    #[display("Unspecified error")]
    UnspecifiedError,
    /// CONNACK, DISCONNECT = 0x81
    #[display("Malformed Packet")]
    MalformedPacket,
    /// CONNACK, DISCONNECT = 0x82
    #[display("Protocol Error")]
    ProtocolError,
    /// CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT = 0x83
    #[display("Implementation specific error")]
    ImplementationSpecificError,
    /// CONNACK = 0x84
    #[display("Unsupported Protocol Version")]
    UnsupportedProtocolVersion,
    /// CONNACK = 0x85
    #[display("Client Identifier not valid")]
    ClientIdentifierNotValid,
    /// CONNACK = 0x86
    #[display("Bad User Name or Password")]
    BadUserNameOrPassword,
    /// CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT = 0x87
    #[display("Not authorized")]
    NotAuthorized,
    /// CONNACK = 0x88
    #[display("Server unavailable")]
    ServerUnavailable,
    /// CONNACK, DISCONNECT = 0x89
    #[display("Server busy")]
    ServerBusy,
    /// CONNACK = 0x8A
    #[display("Banned")]
    Banned,
    /// DISCONNECT = 0x8B
    #[display("Server shutting down")]
    ServerShuttingDown,
    /// CONNACK, DISCONNECT = 0x8C
    #[display("Bad authentication method")]
    BadAuthenticationMethod,
    /// DISCONNECT = 0x8D
    #[display("Keep Alive timeout")]
    KeepAliveTimeout,
    /// DISCONNECT = 0x8E
    #[display("Session taken over")]
    SessionTakenOver,
    /// SUBACK, UNSUBACK, DISCONNECT = 0x8F
    #[display("Topic Filter invalid")]
    TopicFilterInvalid,
    /// CONNACK, PUBACK, PUBREC, DISCONNECT = -0x90
    #[display("Topic name invalid")]
    TopicNameInvalid,
    /// PUBACK, PUBREC, SUBACK, UNSUBACK = 0x91
    #[display("Packet Identidier in use")]
    PacketIdentifierInUse,
    /// PUBREL, PUBCOMP = 0x92
    #[display("Packet Identifier not found")]
    PacketIdentifierNotFound,
    /// DISCONNECT = 0x93
    #[display("Receive Maximum exceeded")]
    ReceiveMaximumExceeded,
    /// DISCONNECT = 0x94
    #[display("Topic Alias invalid")]
    TopicAliasInvalid,
    /// CONNACK, DISCONNECT = 0x95
    #[display("Packet too large")]
    PacketTooLarge,
    /// DISCONNECT = 0x96
    #[display("Message rate too high")]
    MessageRateTooHigh,
    /// CONNACK, PUBACK, PUBREC, SUBACK, DISCONNECT = 0x97
    #[display("Quota exceeded")]
    QuotaExceeded,
    /// DISCONNECT = 0x98
    #[display("Administrative action")]
    AdministrativeAction,
    /// CONNACK, PUBACK, PUBREC, DISCONNECT = 0x99
    #[display("Payload format invalid")]
    PayloadFormatInvalid,
    /// CONNACK, DISCONNECT = 0x9A
    #[display("Retain not supported")]
    RetainNotSupported,
    /// CONNACK, DISCONNECT = 0x9B
    #[display("QoS not supported")]
    QoSNotSupported,
    /// CONNACK, DISCONNECT = 0x9C
    #[display("Use another server")]
    UseAnotherServer,
    /// CONNACK, DISCONNECT = 0x9D
    #[display("Server moved")]
    ServerMoved,
    /// SUBACK, DISCONNECT = 0x9E
    #[display("Shared Subcriptions not supported")]
    SharedSubscriptionsNotSupported,
    /// CONNACK, DISCONNECT = 0x9F
    #[display("Connection rate exceeded")]
    ConnectionRateExceeded,
    /// DISCONNECT = 0xA0
    #[display("Maximum connect time")]
    MaximumConnectTime,
    /// SUBACK, DISCONNECT = 0xA1
    #[display("Subscription Identifiers not supported")]
    SubscriptionIdentifiersNotSupported,
    /// SUBACK, DISCONNECT = 0xA2
    #[display("Wildcard Subscriptions not supported")]
    WildcardSubscriptionsNotSupported,
}


impl From<ReasonCode> for u8 {
    fn from(value: ReasonCode) -> Self {
        match value {
        
            ReasonCode::Success | ReasonCode::NormalDisconnection | ReasonCode::GrantedQoS0 => 0x00,
            ReasonCode::GrantedQoS1 => 0x01,
            ReasonCode::GrantedQoS2 => 0x02,
            ReasonCode::DisconnectWithWillMessage => 0x04,
            ReasonCode::NoMatchingSubscribers => 0x10,
            ReasonCode::NoSubscriptionExisted => 0x11,
            ReasonCode::ContinueAuthentication => 0x18,
            ReasonCode::ReAuthenticate => 0x19,
            ReasonCode::UnspecifiedError => 0x80,
            ReasonCode::MalformedPacket => 0x81,
            ReasonCode::ProtocolError => 0x82,
            ReasonCode::ImplementationSpecificError => 0x83,
            ReasonCode::UnsupportedProtocolVersion => 0x84,
            ReasonCode::ClientIdentifierNotValid => 0x85,
            ReasonCode::BadUserNameOrPassword => 0x86,
            ReasonCode::NotAuthorized => 0x87,
            ReasonCode::ServerUnavailable => 0x88,
            ReasonCode::ServerBusy => 0x89,
            ReasonCode::Banned => 0x8A,
            ReasonCode::ServerShuttingDown => 0x8B,
            ReasonCode::BadAuthenticationMethod => 0x8C,
            ReasonCode::KeepAliveTimeout => 0x8D,
            ReasonCode::SessionTakenOver => 0x8E,
            ReasonCode::TopicFilterInvalid => 0x8F,
            ReasonCode::TopicNameInvalid => 0x90,
            ReasonCode::PacketIdentifierInUse => 0x91,
            ReasonCode::PacketIdentifierNotFound => 0x92,
            ReasonCode::ReceiveMaximumExceeded => 0x93,
            ReasonCode::TopicAliasInvalid => 0x94,
            ReasonCode::PacketTooLarge => 0x95,
            ReasonCode::MessageRateTooHigh => 0x96,
            ReasonCode::QuotaExceeded => 0x97,
            ReasonCode::AdministrativeAction => 0x98,
            ReasonCode::PayloadFormatInvalid => 0x99,
            ReasonCode::RetainNotSupported => 0x9A,
            ReasonCode::QoSNotSupported => 0x9B,
            ReasonCode::UseAnotherServer => 0x9C,
            ReasonCode::ServerMoved => 0x9D,
            ReasonCode::SharedSubscriptionsNotSupported => 0x9E,
            ReasonCode::ConnectionRateExceeded => 0x9F,
            ReasonCode::MaximumConnectTime => 0xA0,
            ReasonCode::SubscriptionIdentifiersNotSupported => 0xA1,
            ReasonCode::WildcardSubscriptionsNotSupported => 0xA2,
            _ => 0x80
        }
    }
}