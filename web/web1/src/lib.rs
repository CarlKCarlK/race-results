#![no_std]
extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn disjoint_intervals(input: Vec<i32>) -> JsValue {
 	let set = "hello";
    let s = set.to_string();
    JsValue::from_str(&s)
}