pub mod stdlib; //* Referring to XMLangEE's stdlib - contains standart library functions */
pub mod randlib;//* XMLangEE's randlib - contains utilities for random number generation */
pub mod mathlib;//* XMLangEE's mathlib - contains mathemathical functions and operations */
pub mod iolib;//* XMLangEE's iolib - contains functions for file I/O */
pub mod cryptolib; //*XMLangEE's cryptolib - contains cryptographic hash functions    */


//? Supers
use super::structures::{
    Interpreter,
    LangError
};

//? Import checker 
pub fn import(name: &str, state: &mut Interpreter) -> Result<(), LangError> {
    return Ok(match name {
        //* included libraries */
        "std" => stdlib::get(state),
        "rand" => randlib::get(state),
        "math" => mathlib::get(state),
        "io" => iolib::get(state),
        "crypto" => cryptolib::get(state),
        name => {
            return Err(
                LangError::RuntimeError(format!("No built-in lib {name}"))
            ); //*! If no included library is matched
        }
    });
}

//* Make function macro */
#[macro_export]
macro_rules! make_func {
    ( $state: ident ; ( $args: literal ) $name: expr => $body: expr ; $( ( $args_l: literal ) $names: expr => $bodies: expr ; )+) => {
        make_func!( $state ; ( $args ) $name => $body ; );
        make_func!( $state ; $( ( $args_l ) $names => $bodies ; )+ );
    };
    ( $state: ident ; ( $args: literal ) $name: expr => $body: expr ; ) => {
        $state.variables.insert(
            $name.into(), 
            Value::Function(
                vec!["".into(); $args],
                FunctionBody::Native(
                    $body
                )
            )
        );
    };
}

pub use make_func;