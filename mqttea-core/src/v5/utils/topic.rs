use crate::v5::commons::error::MQTTError;

pub(crate) fn parse_alias(alias: u16, alias_max: u16) -> Result<u16, MQTTError> {
    if alias == 0 || alias > alias_max {
        return Err(MQTTError::InvalidProperty(
            format!("Topic Alias Must be non-zero and less than or equal to topic alias maximum {alias_max} but got {alias:?}")
        ));
    }
    Ok(alias)
}
