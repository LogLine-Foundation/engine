#![forbid(unsafe_code)]

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptEncodingError {
    StringEncoding,
    ReceiptMustBeObject,
}

impl fmt::Display for ReceiptEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StringEncoding => formatter.write_str("failed to encode JSON string"),
            Self::ReceiptMustBeObject => formatter.write_str("receipt must be a JSON object"),
        }
    }
}

impl std::error::Error for ReceiptEncodingError {}

pub type ReceiptEncodingResult<T> = std::result::Result<T, ReceiptEncodingError>;

pub fn canonical_json(value: &Value) -> ReceiptEncodingResult<String> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(value) => Ok(if *value { "true" } else { "false" }.to_string()),
        Value::Number(number) => Ok(number.to_string()),
        Value::String(value) => {
            serde_json::to_string(value).map_err(|_| ReceiptEncodingError::StringEncoding)
        }
        Value::Array(values) => {
            let mut encoded = String::from("[");

            for (index, item) in values.iter().enumerate() {
                if index > 0 {
                    encoded.push(',');
                }
                encoded.push_str(&canonical_json(item)?);
            }

            encoded.push(']');
            Ok(encoded)
        }
        Value::Object(object) => canonical_object_json(object),
    }
}

pub fn result_hash(result: &Value) -> ReceiptEncodingResult<String> {
    sha256_json(result)
}

pub fn receipt_hash(receipt: &Value) -> ReceiptEncodingResult<String> {
    let mut receipt = receipt.clone();
    let Value::Object(object) = &mut receipt else {
        return Err(ReceiptEncodingError::ReceiptMustBeObject);
    };

    if let Some(Value::Object(hashes)) = object.get_mut("hashes") {
        hashes.remove("receipt_hash");
    }

    sha256_json(&receipt)
}

fn canonical_object_json(object: &Map<String, Value>) -> ReceiptEncodingResult<String> {
    let mut keys: Vec<&String> = object.keys().collect();
    keys.sort();

    let mut encoded = String::from("{");

    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            encoded.push(',');
        }

        let key_json =
            serde_json::to_string(key).map_err(|_| ReceiptEncodingError::StringEncoding)?;
        encoded.push_str(&key_json);
        encoded.push(':');
        encoded.push_str(&canonical_json(&object[*key])?);
    }

    encoded.push('}');
    Ok(encoded)
}

fn sha256_json(value: &Value) -> ReceiptEncodingResult<String> {
    let canonical = canonical_json(value)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!("sha256:{}", hex_lower(&digest)))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}
