use crate::util::structures::*;
use crate::util::Interpreter;

use std::fs::*;

use std::num::Wrapping;

use super::make_func;

pub fn get(state: &mut Interpreter) {
    make_func! {
        state;
        (1) "read" => |args| {
            if let Value::String(ref path) = args[0] {
                if let Ok(s) = read_to_string(path) {
                    return Ok(Some(Value::String(s)));
                } else {
                    return Err(LangError::RuntimeError(format!("Could not open file {} for reading", path)));
                }
            } else {
                return Err(LangError::RuntimeError("Expected string for file path".into()));
            }
        };
        (1) "read-bin" => |args| {
            if let Value::String(ref path) = args[0] {
                if let Ok(s) = read(path) {
                    return Ok(Some(Value::Array(
                        s.iter().map(|chr| Value::Integer(Wrapping(*chr as i64))).collect()
                    )));
                } else {
                    return Err(LangError::RuntimeError(format!("Could not open file {} for reading", path)));
                }
            } else {
                return Err(LangError::RuntimeError("Expected string for file path".into()));
            }
        };
        (2) "write" => |args| {
            if let (Value::String(path), Value::String(data)) = (&args[0], &args[1]) {
                if let Ok(_) = write(path, data) {
                    return Ok(None)
                } else {
                    return Err(LangError::RuntimeError(format!("Could not open file {} for writing", path)));
                }
            } else {
                return Err(LangError::RuntimeError("Expected string for file path and data".into()));
            }
        };
        (2) "write-bin" => |args| {
            if let Value::String(path) = &args[0] {
                if let Value::Array(data) = &args[1] {
                    let bin_data_maybe: Result<Vec<u8>, LangError> = data.iter().map(
                        |v| {
                            if let Value::Integer(i) = v {
                                Ok(i.0 as u8)
                            } else {
                                Err(LangError::RuntimeError("Non-integer found in write array".into()))
                            }
                        }
                    ).collect();
                    let bin_data = bin_data_maybe?;
                    if let Ok(_) = write(path, bin_data) {
                        return Ok(None)
                    } else {
                        return Err(LangError::RuntimeError(format!("Could not open file {} for writing", path)));
                    }
                } else {
                    return Err(LangError::RuntimeError("Expected array for file data".into()));
                }
            } else {
                return Err(LangError::RuntimeError("Expected string for file path".into()));
            }
        };
                // Append to a file (creates if not exists)
        (2) "append" => |args| {
            if let (Value::String(path), Value::String(data)) = (&args[0], &args[1]) {
                use std::fs::OpenOptions;
                use std::io::Write;
                
                let mut file = match OpenOptions::new()
                    .create(true)   // Create if doesn't exist
                    .append(true)   // Append to end
                    .open(path) {
                    Ok(f) => f,
                    Err(e) => return Err(LangError::RuntimeError(
                        format!("Could not open file {} for appending: {}", path, e)
                    )),
                };
                
                match file.write_all(data.as_bytes()) {
                    Ok(_) => Ok(None),
                    Err(e) => Err(LangError::RuntimeError(
                        format!("Failed to append to {}: {}", path, e)
                    )),
                }
            } else {
                Err(LangError::RuntimeError("Expected (path, data) strings".into()))
            }
        };

        // Write with explicit create flag (create(true) is already default for write)
        // But for clarity, add a function that returns error if file exists
        (2) "write_new" => |args| {
            if let (Value::String(path), Value::String(data)) = (&args[0], &args[1]) {
                use std::fs::OpenOptions;
                use std::io::Write;
                
                let mut file = match OpenOptions::new()
                    .write(true)
                    .create_new(true)  // Fails if file exists (like Python 'x')
                    .open(path) {
                    Ok(f) => f,
                    Err(e) => return Err(LangError::RuntimeError(
                        format!("Could not create new file {}: {}", path, e)
                    )),
                };
                
                match file.write_all(data.as_bytes()) {
                    Ok(_) => Ok(None),
                    Err(e) => Err(LangError::RuntimeError(
                        format!("Failed to write to {}: {}", path, e)
                    )),
                }
            } else {
                Err(LangError::RuntimeError("Expected (path, data) strings".into()))
            }
        };

        // Check if a file exists
        (1) "exists" => |args| {
            if let Value::String(path) = &args[0] {
                Ok(Some(Value::Boolean(std::path::Path::new(path).exists())))
            } else {
                Err(LangError::RuntimeError("Expected path string".into()))
            }
        };
    }
}
