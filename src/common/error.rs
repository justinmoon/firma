use crate::ErrorJson;
use serde_json::Value;

#[derive(Debug)]
pub struct Error(pub String);

impl Error {
    pub fn to_json(self) -> Result<Value, Error> {
        let value = ErrorJson { error: self.0 };
        Ok(serde_json::to_value(&value)?)
    }
}

pub fn err<R>(str: &str) -> Result<R, Error> {
    Err(Error(str.into()))
}

pub fn fn_err(str: &str) -> impl Fn() -> Error + '_ {
    move || Error(str.into())
}

pub fn io_err(str: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, str.to_string())
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error(e)
    }
}

macro_rules! impl_error {
    ( $from:ty ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error(err.to_string())
            }
        }
    };
}

impl_error!(bitcoincore_rpc::Error);
impl_error!(&str);
impl_error!(serde_json::error::Error);
impl_error!(std::io::Error);
impl_error!(bitcoin::util::base58::Error);
impl_error!(bitcoin::util::bip32::Error);
impl_error!(base64::DecodeError);
impl_error!(bitcoin::consensus::encode::Error);
impl_error!(std::path::StripPrefixError);
impl_error!(qrcode::types::QrError);
impl_error!(bitcoin::util::key::Error);
impl_error!(bitcoin::secp256k1::Error);
impl_error!(bitcoin::util::psbt::Error);
impl_error!(bitcoin::util::address::Error);
impl_error!(hex::FromHexError);
impl_error!(std::env::VarError);
