use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the payload type `{0}` is invalid")]
    InvalidPayloadType(String),
    #[error("payload type not match, (expected {expected:?}, found {found:?})")]
    PayloadTypeNotMatch {
        expected: PayloadType,
        found: PayloadType,
    },
    #[error("bincode error: {0}")]
    BincodeError(bincode::Error),
    #[error("json error: {0}")]
    JsonError(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodeType {
    Bincode, // 直接基于内存的序列化方案
    Json,    // 基于JSON的序列化方案
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadType(String);

impl From<PayloadType> for String {
    fn from(val: PayloadType) -> Self {
        val.0
    }
}

impl TryFrom<String> for PayloadType {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,15}$").unwrap());
        if RE.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidPayloadType(value))
        }
    }
}

impl Display for PayloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 一个可序列化的结构体必须实现该trait
pub trait Payload: Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn payload_type() -> PayloadType;

    fn try_encode(&self, encode_type: EncodeType) -> Result<Vec<u8>> {
        match encode_type {
            EncodeType::Bincode => bincode::serialize(self).map_err(Error::BincodeError),
            EncodeType::Json => serde_json::to_vec(self).map_err(Error::JsonError),
        }
    }
    fn try_decode(encode_type: EncodeType, bytes: &[u8]) -> Result<Self> {
        match encode_type {
            EncodeType::Bincode => bincode::deserialize(bytes).map_err(Error::BincodeError),
            EncodeType::Json => serde_json::from_slice(bytes).map_err(Error::JsonError),
        }
    }
    fn to_raw_payload(&self, encode_type: EncodeType) -> Result<RawPayload> {
        Ok(RawPayload {
            encode_type,
            payload_type: Self::payload_type(),
            raw: self.try_encode(encode_type)?,
        })
    }
    fn from_raw_payload(raw_payload: &RawPayload) -> Result<Self> {
        if raw_payload.payload_type == Self::payload_type() {
            Ok(Self::try_decode(raw_payload.encode_type, &raw_payload.raw)?)
        } else {
            Err(Error::PayloadTypeNotMatch {
                expected: Self::payload_type(),
                found: raw_payload.payload_type.clone(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawPayload {
    encode_type: EncodeType,
    payload_type: PayloadType,
    raw: Vec<u8>,
}

impl RawPayload {
    pub fn try_decode<T: Payload>(&self) -> Result<T> {
        T::from_raw_payload(self)
    }

    pub fn try_encode<T: Payload>(val: &T, encode_type: EncodeType) -> Result<Self> {
        T::to_raw_payload(val, encode_type)
    }
}
