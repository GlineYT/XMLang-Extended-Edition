use crate::util::structures::*;
use crate::util::FunctionBody;
use crate::util::Interpreter;

use std::io;
use std::io::Write;
use std::num::Wrapping;
use std::thread;
use std::time::{Duration, SystemTime};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use radix_fmt::radix;

use super::make_func;


pub fn get(state: &mut Interpreter) {
    make_func!(
        state;
        //* Print function */
        (1) "print" => |values| {
            print!("{}", values[0]);
            Ok(None)
        };

        //* Print a newline aswell */
        (1) "println" => |values| {
            println!("{}", values[0]);
            Ok(None)
        };

        //* Function to flush the standart output */
        (0) "flush" => |_| {
            if let Err(_) = io::stdout().flush() {
                return Err(LangError::RuntimeError(
                    "Failed to flush stdout".into()
                ));
            }
            Ok(None)
        };

        //* Iterate over a range, similar to python's "for i in range" */
        (3) "range" => |values| {
            if let (Value::Integer(mut a), Value::Integer(mut b), Value::Integer(mut step))
                = (&values[0], &values[1], &values[2]) {
                if step.0 == 0 {
                    return Err(LangError::RuntimeError(
                        "Can't make a range with 0 step".into()
                    ))
                }
                Ok(Some(Value::Array(
                    if step.0 < 0 {
                        (b.0+1 ..= a.0).rev().step_by(-step.0 as usize)
                        .map(|i| Value::Integer(Wrapping(i)))
                        .collect()
                    } else {
                        (a.0 .. b.0).step_by(step.0 as usize)
                        .map(|i| Value::Integer(Wrapping(i)))
                        .collect()
                    }
                )))
            } else {
                return Err(LangError::RuntimeError(
                    "Can't create a non-integer range".into()
                ))
            }
        };

        //* Turns array into string */
        (1) "join" => |values| {
            if let Value::Array(arr) = &values[0] {
                let mut out = String::new();
                for v in arr {
                    out.push_str(v.to_string().as_str());
                }
                return Ok(Some(Value::String(out)));
            }
            Err(LangError::RuntimeError(
                "Expected array to join".into()
            ))
        };

        //* Array concatenations */
    (2) "concat" => |values| {
        if let (Value::Array(arr1), Value::Array(arr2)) = (&values[0], &values[1]) {
            let mut result = arr1.clone();
            result.extend(arr2.clone());
            Ok(Some(Value::Array(result)))
        } else {
            Err(LangError::RuntimeError("Expected two arrays".into()))
        }
    };

        //* Simple text splitting function */
        (2) "split" => |values| {
                if let (Value::String(s), Value::String(delim)) = (&values[0], &values[1]) {
                    let parts: Vec<Value> = s.split(delim).map(|part| Value::String(part.to_string())).collect();
                    Ok(Some(Value::Array(parts)))
                } else {
                    Err(LangError::RuntimeError("Expected two strings: string to split and delimiter".into()))
                }
            };
        
        //* Array splitting */
            (2) "split_at" => |values| {
                if let (Value::Array(arr), Value::Integer(index)) = (&values[0], &values[1]) {
                    let idx = index.0 as usize;
                    if idx > arr.len() {
                        return Err(LangError::RuntimeError(
                            format!("Split index {} out of bounds (len {})", idx, arr.len())
                        ));
                    }
                    let left = Value::Array(arr[0..idx].to_vec());
                    let right = Value::Array(arr[idx..].to_vec());
                    let result = Value::Array(vec![left, right]);
                    Ok(Some(result))
                } else {
                    Err(LangError::RuntimeError(
                        "Expected array and integer index".into()
                    ))
                }
            };

            //* Modify array at index (returns new array) */
            (3) "array_modify" => |values| {
                if let (Value::Array(arr), Value::Integer(index), new_value) = (&values[0], &values[1], &values[2]) {
                    let idx = index.0 as usize;
                    if idx >= arr.len() {
                        return Err(LangError::RuntimeError(
                            format!("Index {} out of bounds for array of length {}", idx, arr.len())
                        ));
                    }
                    let mut result = arr.clone();
                    result[idx] = new_value.clone();
                    Ok(Some(Value::Array(result)))
                } else {
                    Err(LangError::RuntimeError(
                        "Expected (array, integer index, new value)".into()
                    ))
                }
            };

            //* Remove element at index (returns new array) */
            (2) "array_remove" => |values| {
                if let (Value::Array(arr), Value::Integer(index)) = (&values[0], &values[1]) {
                    let idx = index.0 as usize;
                    if idx >= arr.len() {
                        return Err(LangError::RuntimeError(
                            format!("Index {} out of bounds for array of length {}", idx, arr.len())
                        ));
                    }
                    let mut result = arr.clone();
                    result.remove(idx);
                    Ok(Some(Value::Array(result)))
                } else {
                    Err(LangError::RuntimeError(
                        "Expected (array, integer index)".into()
                    ))
                }
            };

        //* Substrings */
        (3) "substr" => |args| {
            if let (Value::String(s), Value::Integer(start), Value::Integer(end)) = (&args[0], &args[1], &args[2]) {
                let s_len = s.chars().count();
                let start_idx = start.0 as usize;
                let end_idx = end.0 as usize;
                
                if start_idx > s_len || end_idx > s_len || start_idx > end_idx {
                    return Err(LangError::RuntimeError(
                        format!("Invalid substring bounds: start={}, end={}, len={}", start_idx, end_idx, s_len)
                    ));
                }
                
                let result: String = s.chars().skip(start_idx).take(end_idx - start_idx).collect();
                Ok(Some(Value::String(result)))
            } else {
                Err(LangError::RuntimeError(
                    "Expected string and two integers (start, end)".into()
                ))
            }
        };

        //* Get the lenght of a given iterable */
        (1) "len" => |values| { 
            return Ok(Some(Value::Integer(Wrapping(
                match &values[0] {
                    Value::Array(arr) => arr.len() as i64,
                    Value::Dictionary(dict) => dict.0.len() as i64,
                    Value::Set(set) => set.0.len() as i64,
                    Value::String(string) => string.chars().count() as i64,
                    _ => return Err(
                        LangError::RuntimeError(
                            "Tried to get the length of an unsized value".into()
                        )
                    )
                }
            ))))
        };

        //*Recieve input from standart input */
        (0) "input" => |_| {
                let mut out = String::new();
                match io::stdin().read_line(&mut out) {
                    Ok(_) => {
                        let trimmed = out.trim().to_string();
                        Ok(Some(Value::String(trimmed)))
                    },
                    Err(e) => Err(LangError::RuntimeError(
                        format!("Error reading input: {}", e).into()
                    ))
                }
            };

        //* Turn a ASCII int into a character */
        (1) "chr" => |args| {
            if let Value::Integer(i) = args[0] {
                if i.0 < 0 {
                    return Err(LangError::RuntimeError("Can't get a negative character".into()));
                }
                if let Some(c) = char::from_u32(i.0 as u32) {
                    let o = c.to_string();
                    return Ok(Some(Value::String(o)));
                } else {
                    return Err(LangError::RuntimeError(format!("Invalid character: {i}")));
                }
            } else {
                return Err(LangError::RuntimeError("Expected integer for call to chr".into()));
            }
        };

        //* Turn a character into a codepoint number */
        (1) "ord" => |args| {
            if let Value::String(s) = &args[0] {
                if s.chars().count() == 1 {
                    let c = s.chars().nth(0).unwrap() as i64;
                    return Ok(Some(Value::Integer(Wrapping(c))));
                } else {
                    return Err(LangError::RuntimeError(
                        format!("Expected string of length 1 for call to ord, got length {}", s.chars().count())
                    ))
                }
            } else {
                return Err(LangError::RuntimeError("Expected string for call to ord".into()));
            }
        };

        //* Base conversion function */
        (2) "radix" => |args| {
            if let Value::Integer(b) = &args[1] {
                if !(2..=36).contains(&b.0) {
                    return Err(LangError::RuntimeError(format!("Base for radix should be in range [2, 36], got {}", b)));
                }
                let b = b.0 as u32;
                match &args[0] {
                    Value::Integer(i) => {
                        return Ok(Some(Value::String(
                            format!("{}", radix(i.0, b as u8))
                        )))
                    },
                    Value::String(s) => {
                        if let Ok(v) = u32::from_str_radix(s.as_str(), b) {
                            return Ok(Some(Value::Integer(Wrapping(v as i64))));
                        } else {
                            return Err(LangError::RuntimeError(format!("Failed to convert {} to int of base {}", s, b)));
                        }
                    },
                    _ => {
                        return Err(LangError::RuntimeError("Expected string or integer for first value of radix".into()));
                    }
                }
            } else {
                return Err(LangError::RuntimeError("Expected an integer for base of radix".into()));
            }
        };

        //* Sleep function */
        (1) "sleep" => |args| {
            let t: Duration;
            if let Value::Integer(i) = args[0] {
                let i = if i.0 < 0 {0u64} else {i.0 as u64};
                t = Duration::from_secs(i);
            } else if let Value::Float(f) = args[0] {
                let f = if *f < 0.0 {0.0} else {*f};
                if !f.is_finite() {
                    return Err(LangError::RuntimeError("Non-finite value given for sleep duration".into()));
                }
                t = Duration::from_secs_f64(f);
            } else {
                return Err(LangError::RuntimeError("Expected numeric value for sleep duration".into()));
            }
            thread::sleep(t);
            Ok(None)
        };

        //* Get UNIX timestamp */
        (0) "time" => |_| {
            let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_err(
                |_| LangError::RuntimeError("Current system time is before unix epoch (???)".into())
            )?;
            Ok(Some(Value::Float(HashableFloat(t.as_secs_f64()))))
        };

        //* Get a given variable type */
        (1) "type" => |args| {
            Ok(Some(Value::String(match &args[0] {
                Value::Integer(_) => "int",
                Value::Float(_) => "float",
                Value::Boolean(_) => "bool",
                Value::String(_) => "str",
                Value::Array(_) => "arr",
                Value::Dictionary(_) => "dict",
                Value::Set(_) => "set",
                Value::Function(_, __) => "func",
                Value::Null => "null",
                Value::Break => "break",
                Value::Continue => "continue"
            }.into())))
        };

        //* Get a hash value of a input 
        //*! (this is NOT a cryptographic hash) 
    
        (1) "hash" => |args| {
            let mut hasher = DefaultHasher::new();
            args[0].hash(&mut hasher);
            Ok(Some(Value::Integer(
                Wrapping(hasher.finish() as i64)
            )))
        };

            //* Merge two dictionaries (second overwrites first on key conflicts)
    (2) "dict_merge" => |args| {
        if let (Value::Dictionary(dict1), Value::Dictionary(dict2)) = (&args[0], &args[1]) {
            let mut result = dict1.clone();
            for (k, v) in dict2.iter() {
                result.insert(k.clone(), v.clone());
            }
            Ok(Some(Value::Dictionary(result)))
        } else {
            Err(LangError::RuntimeError("Expected two dictionaries".into()))
        }
    };

    //* Set key-value pair (returns new dict)
    (3) "dict_set" => |args| {
        if let (Value::Dictionary(dict), key, value) = (&args[0], &args[1], &args[2]) {
            let mut result = dict.clone();
            result.insert(key.clone(), value.clone());
            Ok(Some(Value::Dictionary(result)))
        } else {
            Err(LangError::RuntimeError("Expected (dict, key, value)".into()))
        }
    };

    //* Get value by key (returns null if not found)
    (2) "dict_get" => |args| {
        if let (Value::Dictionary(dict), key) = (&args[0], &args[1]) {
            Ok(dict.get(key).cloned())
        } else {
            Err(LangError::RuntimeError("Expected (dict, key)".into()))
        }
    };

    //* Remove key-value pair (returns new dict)
    (2) "dict_remove" => |args| {
        if let (Value::Dictionary(dict), key) = (&args[0], &args[1]) {
            let mut result = dict.clone();
            result.remove(key);
            Ok(Some(Value::Dictionary(result)))
        } else {
            Err(LangError::RuntimeError("Expected (dict, key)".into()))
        }
    };

    //* Check if key exists
    (2) "dict_has" => |args| {
        if let (Value::Dictionary(dict), key) = (&args[0], &args[1]) {
            Ok(Some(Value::Boolean(dict.contains_key(key))))
        } else {
            Err(LangError::RuntimeError("Expected (dict, key)".into()))
        }
    };

    //* Get all keys
    (1) "dict_keys" => |args| {
        if let Value::Dictionary(dict) = &args[0] {
            let keys: Vec<Value> = dict.keys().cloned().collect();
            Ok(Some(Value::Array(keys)))
        } else {
            Err(LangError::RuntimeError("Expected dictionary".into()))
        }
    };

    //* Get all values
    (1) "dict_values" => |args| {
        if let Value::Dictionary(dict) = &args[0] {
            let values: Vec<Value> = dict.values().cloned().collect();
            Ok(Some(Value::Array(values)))
        } else {
            Err(LangError::RuntimeError("Expected dictionary".into()))
        }
    };

    //* Get all key-value pairs as array of [key, value] arrays
    (1) "dict_items" => |args| {
        if let Value::Dictionary(dict) = &args[0] {
            let items: Vec<Value> = dict.iter()
                .map(|(k, v)| Value::Array(vec![k.clone(), v.clone()]))
                .collect();
            Ok(Some(Value::Array(items)))
        } else {
            Err(LangError::RuntimeError("Expected dictionary".into()))
        }
    };
    );
}
