use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) fn stable_config_id(prefix: &str, value: &impl Serialize) -> String {
    let bytes =
        serde_json::to_vec(value).expect("adapter config identity serialization is infallible");
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String is infallible");
    }
    format!("{prefix}:sha256:{hex}")
}
