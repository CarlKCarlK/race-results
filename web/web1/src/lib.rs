extern crate alloc;
use race_results::find_matches;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn member_match(members: &str, results: &str, _with_city: bool) -> JsValue {
    let s = find_matches(members.lines(), results.lines(), results.lines());
    let s = s.join("\n");
    JsValue::from_str(&s)
}
