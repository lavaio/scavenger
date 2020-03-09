use bytes::Buf;
use reqwest::r#async::Chunk;
use serde::de::{self, DeserializeOwned};
use std::fmt;

//
// address1
// nonce
// target deadline(v / baseTarget = deadline)
// targetHeight
#[derive(Debug, Clone, Serialize)]
pub struct SubmitNonceParams(pub String, pub String, pub Option<u64>, pub u64, pub u64);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitNonceRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Option<SubmitNonceParams>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMiningInfoRequest<'a> {
    pub request_type: &'a str,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitNonceResult {
    pub accept: Option<bool>,
    pub plotid: Option<String>,
    pub deadline: Option<u64>,
    pub targetdeadline: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitNonceResponse {
    pub result: SubmitNonceResult,
    pub error: Option<String>,
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningInfoResult {
    pub generation_signature: String,

    #[serde(deserialize_with = "from_str_or_int")]
    pub base_target: u64,

    #[serde(deserialize_with = "from_str_or_int")]
    pub height: u64,

    #[serde(
        default = "default_target_deadline",
        deserialize_with = "from_str_or_int"
    )]
    pub target_deadline: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningInfoResponse {
    pub result: MiningInfoResult,
    pub error: Option<String>,
    pub id: String,
}

fn default_target_deadline() -> u64 {
    std::u64::MAX
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PoolErrorWrapper {
    error: PoolError,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug)]
pub enum FetchError {
    Http(reqwest::Error),
    Pool(PoolError),
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<PoolError> for FetchError {
    fn from(err: PoolError) -> FetchError {
        FetchError::Pool(err)
    }
}

// MOTHERFUCKING pool
fn from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> de::Visitor<'de> for StringOrIntVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or int")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<u64>().map_err(de::Error::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

pub fn parse_json_result<T: DeserializeOwned>(body: &Chunk) -> Result<T, PoolError> {
    match serde_json::from_slice(body.bytes()) {
        Ok(x) => Ok(x),
        _ => match serde_json::from_slice::<PoolErrorWrapper>(body.bytes()) {
            Ok(x) => Err(x.error),
            _ => {
                let v = body.to_vec();
                Err(PoolError {
                    code: 0,
                    message: String::from_utf8_lossy(&v).to_string(),
                })
            }
        },
    }
}
