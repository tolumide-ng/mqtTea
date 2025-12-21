use mqttea_macros::FromU8;

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromU8, PartialEq, Eq)]
pub enum SubAckReasonCode {
    /// The subscription is accepted and the maximum Qos sent will be QoS1 (This might be lower than requested)
    GrantedQos0 = 0,
    /// The Subscription is accepted and the maximum QoS sent will be QoS1 (This might be lower than requested)
    GrantedQoS1 = 1,
    /// The subscription is accepted and any received QoS will be sent to this subscription
    GrantedQoS2 = 2,
    UnspecifiedError = 128,
    /// Subscribe packet is valid, but the server does not accept it
    ImplementationSpecificError = 131,
    NotAuhtorized = 135,
    TopicFilterInvalid = 143,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    SharedSubscriptionNotSupported = 158,
    SubscriptionIdentifiersNotSupported = 161,
    WildCardSubscriptionNotSupported = 162,
}
