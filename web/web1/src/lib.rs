#![no_std]
extern crate alloc;
use alloc::string::ToString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn member_match(members: &str, results: &str, with_city: bool) -> JsValue {
    let set = "hello";
    let s = set.to_string();
    JsValue::from_str(&s)
}
