use crate::util::structures::*;
use crate::util::Interpreter;
use super::make_func;

// External crates
use md5;  // MD5 hashing
use sha2::{Sha256, Sha512, Digest};  // SHA256 and SHA512
use bcrypt;  // Password hashing
use hmac::{Hmac, Mac};
use sha2::Sha256 as Sha256Hash;  // For HMAC-SHA256

type HmacSha256 = Hmac<Sha256Hash>;

pub fn get(state: &mut Interpreter) {
    make_func! {
        state;
        
        // Main dispatcher for cryptographic hashing
        (2) "crypto_hash" => |args| {
            if let (Value::String(algorithm), Value::String(input)) = (&args[0], &args[1]) {
                let result = match algorithm.as_str() {
                    "md5" => {
                        let digest = md5::compute(input.as_bytes());
                        Some(Value::String(format!("{:x}", digest)))
                    }
                    "sha256" => {
                        let mut hasher = Sha256::new();
                        hasher.update(input.as_bytes());
                        let digest = hasher.finalize();
                        Some(Value::String(format!("{:x}", digest)))
                    }
                    "sha512" => {
                        let mut hasher = Sha512::new();
                        hasher.update(input.as_bytes());
                        let digest = hasher.finalize();
                        Some(Value::String(format!("{:x}", digest)))
                    }
                    "hmac_sha256" => {
                        // Note: This expects key in the input? Actually separate function needed
                        return Err(LangError::RuntimeError(
                            "Use crypto_hmac for HMAC-SHA256".into()
                        ));
                    }
                    _ => {
                        return Err(LangError::RuntimeError(
                            format!("Unknown algorithm: {}. Supported: md5, sha256, sha512", algorithm)
                        ));
                    }
                };
                Ok(result)
            } else {
                Err(LangError::RuntimeError(
                    "Expected (algorithm string, input string)".into()
                ))
            }
        };
        
        // HMAC-SHA256 (separate because it needs a key)
        (3) "crypto_hmac" => |args| {
            if let (Value::String(algorithm), Value::String(key), Value::String(input)) = (&args[0], &args[1], &args[2]) {
                match algorithm.as_str() {
                    "sha256" => {
                        let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                            .map_err(|e| LangError::RuntimeError(format!("HMAC error: {}", e)))?;
                        mac.update(input.as_bytes());
                        let result = mac.finalize();
                        let bytes = result.into_bytes();
                        Ok(Some(Value::String(format!("{:x}", bytes))))
                    }
                    _ => {
                        Err(LangError::RuntimeError(
                            format!("Unknown HMAC algorithm: {}. Supported: sha256", algorithm)
                        ))
                    }
                }
            } else {
                Err(LangError::RuntimeError(
                    "Expected (algorithm string, key string, input string)".into()
                ))
            }
        };
        
        // bcrypt hash (for passwords)
        (2) "bcrypt_hash" => |args| {
            if let (Value::String(password), Value::Integer(cost)) = (&args[0], &args[1]) {
                let cost_u32 = cost.0 as u32;
                if cost_u32 < 4 || cost_u32 > 31 {
                    return Err(LangError::RuntimeError(
                        "Cost must be between 4 and 31".into()
                    ));
                }
                match bcrypt::hash(password, cost_u32) {
                    Ok(hashed) => Ok(Some(Value::String(hashed))),
                    Err(e) => Err(LangError::RuntimeError(format!("bcrypt error: {}", e))),
                }
            } else {
                Err(LangError::RuntimeError(
                    "Expected (password string, cost integer)".into()
                ))
            }
        };
        
        // bcrypt verify (compare password against stored hash)
        (2) "bcrypt_verify" => |args| {
            if let (Value::String(password), Value::String(hash_str)) = (&args[0], &args[1]) {
                match bcrypt::verify(password, hash_str) {
                    Ok(matches) => Ok(Some(Value::Boolean(matches))),
                    Err(e) => Err(LangError::RuntimeError(format!("bcrypt error: {}", e))),
                }
            } else {
                Err(LangError::RuntimeError(
                    "Expected (password string, hash string)".into()
                ))
            }
        };
    }
}