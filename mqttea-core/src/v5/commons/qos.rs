use mqttea_macros::FromU8;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, FromU8)]
pub enum QoS {
    #[default]
    Zero = 0,
    One = 1,
    Two = 2,
}