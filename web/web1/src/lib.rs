extern crate alloc;
use race_results::find_matches;
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn member_match(members: &str, race_results: &str, _with_city: bool) -> JsValue {
    // cmk using catch_unwind isn't nice
    // cmk the work 'result' is used in two different ways here
    let function_result = panic::catch_unwind(|| {
        find_matches(members.lines(), race_results.lines(), race_results.lines())
    });
    let s = match function_result {
        Ok(match_list) => match_list.join("\n"),
        Err(panic) => format!("Error: {:?}", panic),
    };
    JsValue::from_str(&s)
}

// cmk what if say "match with city" before uploading a file
// cmk when match results are empty, show something
// cmk should we support both *.txt and *.tsv (and *.csv?)
// cmk0 catching panics isn't working
// cmk move the match buttons to be close to Results
// cmk0 remove the alerts