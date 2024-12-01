pub enum Error {
    PayloadTypeNotMatch {
        expect: PayloadType,
        actual: PayloadType,
    },
    BincodeError(bincode::Error),
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

// 一个可序列化的结构体必须实现该trait
pub trait Payload: Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn payload_type() -> PayloadType;

    fn try_encode(&self, encode_type: EncodeType) -> Result<Vec<u8>> {
        match encode_type {
            EncodeType::Bincode => bincode::serialize(self).map_err(Error::BincodeError),
            EncodeType::Json => serde_json::to_vec(self).map_err(Error::JsonError),
        }
    }
    fn try_decode(encode_type: EncodeType, bytes: Vec<u8>) -> Result<Self> {
        match encode_type {
            EncodeType::Bincode => bincode::deserialize(&bytes).map_err(Error::BincodeError),
            EncodeType::Json => serde_json::from_slice(&bytes).map_err(Error::JsonError),
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
    pub fn try_decode<T: Payload>(self) -> Result<T> {
        if self.payload_type == T::payload_type() {
            return Err(Error::PayloadTypeNotMatch {
                expect: T::payload_type(),
                actual: self.payload_type,
            });
        }
        T::try_decode(self.encode_type, self.raw)
    }

    pub fn try_encode<T: Payload>(val: T, encode_type: EncodeType) -> Result<Self> {
        let raw = val.try_encode(encode_type)?;
        Ok(Self {
            encode_type,
            raw,
            payload_type: T::payload_type(),
        })
    }
}
