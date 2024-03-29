extern crate alloc;
use race_results::{Config, IncludeCity, SAMPLE_MEMBERS_STR, SAMPLE_RESULTS_STR};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn member_match(members: &str, race_results: &str, include_city: u8) -> JsValue {
    let include_city = match IncludeCity::try_from(include_city) {
        Ok(include_city) => include_city,
        Err(panic) => return JsValue::from_str(format!("Error: {:?}", panic).as_str()),
    };

    // cmk the word 'result' is used in two different ways here
    let function_result = Config {
        // threshold_probability: 0.0,
        // override_results_count: Some(1081),
        ..Config::default()
    }
    .find_matches(members.lines(), race_results.lines(), include_city);
    let s = match function_result.as_deref() {
        // cmk make this 1% configurable
        Ok([]) => "No matches found above probability 1%".to_string(),
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
// cmk is there way (and should) for javascript to download the big file and pass it over.
// cmk matching on a single letter, like "j" seems too highly weighted.
// cmk choosing the results file is ignored.
