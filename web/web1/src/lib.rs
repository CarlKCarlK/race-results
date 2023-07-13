extern crate alloc;
use race_results::{Config, SAMPLE_MEMBERS_STR, SAMPLE_RESULTS_STR};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn member_match(members: &str, race_results: &str, include_city: bool) -> JsValue {
    // cmk the work 'result' is used in two different ways here
    let function_result = Config {
        // threshold_probability: 0.0,
        // override_results_count: Some(1081),
        ..Config::default()
    }
    .find_matches(
        members.lines(),
        race_results.lines(),
        race_results.lines(),
        include_city,
    );
    let s = match function_result {
        Ok(match_list) => match_list.join("\n"),
        Err(panic) => format!("Error: {:?}", panic),
    };
    JsValue::from_str(&s)
}

#[wasm_bindgen]
pub fn sample_members() -> JsValue {
    JsValue::from_str(&SAMPLE_MEMBERS_STR)
}

#[wasm_bindgen]
pub fn sample_results() -> JsValue {
    JsValue::from_str(&SAMPLE_RESULTS_STR)
}

// cmk what if say "match with city" before uploading a file
// cmk0 when match results are empty, show something
// cmk should there be an hour glass or something while the results are being computed?
// cmk is there way (and should) for javascript to download the big file and pass it over.
// cmk0 does it do the right thing if a name is blank?
