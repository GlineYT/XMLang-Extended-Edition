use crate::util::Interpreter;
use crate::util::structures::*;

use rand::prelude::*;
use uuid::Uuid;  

use super::make_func;

pub fn get(state: &mut Interpreter) {
    make_func! {
        state;

        //* Random number between 0 and 1 */
        (0) "rand" => |_| {
            Ok(Some(Value::Float(HashableFloat(rand::random()))))
        };

        //* GUID generation (V4)*/
        (0) "guid" => |_| {  
            let id = Uuid::new_v4();
            Ok(Some(Value::String(id.to_string())))
        };

        //* Random string, base 2 to 36  */
        (2) "rand_str" => |values| {  
            if let (Value::Integer(length), Value::Integer(base)) = (&values[0], &values[1]) {
                let len = length.0 as usize;
                let base = base.0 as u32;
                
                if base < 2 || base > 36 {
                    return Err(LangError::RuntimeError(
                        format!("Base must be between 2 and 36, got {}", base)
                    ));
                }
                
                let chars: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz"
                    .chars()
                    .take(base as usize)
                    .collect();
                
                let mut result = String::with_capacity(len);
                for _ in 0..len {
                    let idx = (rand::random::<f64>() * base as f64) as usize;
                    result.push(chars[idx]);
                }
                
                Ok(Some(Value::String(result)))
            } else {
                Err(LangError::RuntimeError(
                    "Expected two integers: length and base (2-36)".into()
                ))
            }
        };
        
        //* Shuffling function */
        (1) "shuffle" => |values| {
            let arr = values[0].clone();
            if let Value::Array(mut arr) = arr {
                arr.shuffle(&mut rand::thread_rng());
                Ok(Some(Value::Array(arr)))
            } else {
                Err(LangError::RuntimeError(
                    "Expected array to shuffle".into()
                ))
            }
        };

        //* Random choice function */
        (1) "choice" => |values| {
            let arr = values[0].clone();
            if let Value::Array(arr) = arr {
                Ok(arr.choose(&mut rand::thread_rng()).cloned())
            } else {
                Err(LangError::RuntimeError(
                    "Expected array to get choice from".into()
                ))
            }
        };

        //* Random sample */
        (1) "sample" => |values| {
            if let Value::Integer(count) = values[0] {
                let mut out = Vec::new();
                for _ in 0..count.0 {
                    out.push(Value::Float(HashableFloat(rand::random())));
                }
                Ok(Some(Value::Array(out)))
            } else {
                Err(LangError::RuntimeError(
                    "Expected integer dimension of sample".into()
                ))
            }
        };
    }
}