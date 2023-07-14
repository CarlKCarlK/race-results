#![cfg(target_arch = "wasm32")]
#![allow(clippy::print_literal)]

use race_results::{
    delta_many_names, delta_one, delta_one_name, log_odds, Token, TokenToCoincidence,
};
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn notebook() {
    // Random line is about Robert

    let prob_member_in_race = 0.01;
    let result_count = 1000;

    let prior_prob = prob_member_in_race / result_count as f32;
    // println!("{prior_prob:.5}"); // => 0.00001

    // Our Robert leads to "Robert"

    let prob_right = 0.6;
    // println!("{prob_right:.5}");

    // Someone else leads to "Robert"

    let name_to_conincidence = TokenToCoincidence::default_names();
    let prob_coincidence = name_to_conincidence.prob(&Token::new("ROBERT"));
    // println!("{prob_coincidence}"); // => 0.03143

    // "Robert" is from Robert

    let prior_points = log_odds(prior_prob);
    // println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");
    assert_eq!(prior_points, -11.512915);

    let delta_points = (prob_right / prob_coincidence).ln();
    assert_eq!(delta_points, 2.9491668);

    let post_points = prior_points + delta_points;
    // println!(
    //     "post: {post_points:.2} points, {:.5} probability",
    //     prob(post_points)
    // );
    assert_eq!(post_points, -8.563747);

    // No "Robert", but still from Robert

    // println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");

    let delta_points = ((1.0 - prob_right) / (1.0 - prob_coincidence)).ln();
    assert_eq!(delta_points, -0.88435626);

    let post_points = prior_points + delta_points;
    assert_eq!(post_points, -12.397271);

    // "Robert" and "Scott" is from Robert Scott.

    // println!(
    //     "prior: {:.2} points, {:.5} probability",
    //     prior_points, prior_prob
    // );

    let first_name_points = (prob_right / name_to_conincidence.prob(&Token::new("ROBERT"))).ln();
    // println!("first_name: {:.2} points", first_name_points);

    let last_name_points = (prob_right / name_to_conincidence.prob(&Token::new("SCOTT"))).ln();
    // println!("last_name: {:.2} points", last_name_points);

    let post_points = prior_points + first_name_points + last_name_points;
    // println!(
    //     "post: {:.2} points, {:.5} probability",
    //     post_points,
    //     prob(post_points)
    // );
    assert_eq!(post_points, -3.8642664);

    let first_name = "CHELLIE";
    let last_name = "PINGREE";

    // println!(
    //     "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
    //     format!("contains_{first_name}"),
    //     format!("contains_{last_name}"),
    //     "prior prob",
    //     "prior points",
    //     "first name points",
    //     "last name points",
    //     "post points",
    //     "post prob"
    // );

    for contains_first_name in [false, true].iter() {
        let first_name_points = delta_one_name(
            *contains_first_name,
            &Token::new(first_name),
            prob_right,
            &name_to_conincidence,
        );
        for contains_last_name in [false, true].iter() {
            let last_name_points = delta_one_name(
                *contains_last_name,
                &Token::new(last_name),
                prob_right,
                &name_to_conincidence,
            );
            let post_points = prior_points + first_name_points + last_name_points;

            // println!(
            //     "{}\t{}\t{:.6}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.6}",
            //     contains_first_name,
            //     contains_last_name,
            //     prior_prob,
            //     prior_points,
            //     first_name_points,
            //     last_name_points,
            //     post_points,
            //     prob(post_points),
            // );
            match (contains_first_name, contains_last_name) {
                (false, false) => assert_eq!(post_points, -13.345492),
                (false, true) => assert_eq!(post_points, -0.40545368),
                (true, false) => assert_eq!(post_points, 0.93899846),
                (true, true) => assert_eq!(post_points, 13.879037),
            }
        }
    }

    // "Bob" is from Robert

    // println!(
    //     "prior: {:.2} points, {:.5} probability",
    //     prior_points, prior_prob
    // );

    let mut first_name_points: f32 = f32::NEG_INFINITY;
    for (name, prob_right, contains) in [
        ("ROBERT", 0.50, true),
        ("BOB", 0.05, true),
        ("ROB", 0.05, false),
    ]
    .iter()
    {
        let name = &Token::new(name);
        let some_first_name_points =
            delta_one_name(*contains, name, *prob_right, &name_to_conincidence);
        println!("\t{:?}: {:.2} points", name, some_first_name_points);
        first_name_points = first_name_points.max(some_first_name_points);
    }
    // println!("first_name: {:.2} points", first_name_points);
    assert_eq!(first_name_points, 4.50986);

    let last_name_points = delta_one_name(
        true,
        &Token::new("SCOTT"),
        prob_right,
        &name_to_conincidence,
    );
    // println!("last_name: {:.2} points", last_name_points);
    assert_eq!(last_name_points, 4.699481);

    let post_points = prior_points + first_name_points + last_name_points;
    // println!(
    //     "post: {:.2} points, {:.5} probability",
    //     post_points,
    //     prob(post_points)
    // );
    assert_eq!(post_points, -2.3035736);

    // "Bellevue" refers to Robert Scott's town.
    // println!(
    //     "prior: {:.2} points, {:.5} probability",
    //     prior_points, prior_prob
    // );

    let first_name_points = delta_many_names(
        [true, true, false].into_iter(),
        &[
            &Token::new("ROBERT"),
            &Token::new("BOB"),
            &Token::new("ROB"),
        ],
        [0.50, 0.05, 0.05].into_iter(),
        &name_to_conincidence,
    );
    assert_eq!(first_name_points, 4.50986);

    let last_name_points = delta_one_name(
        true,
        &Token::new("SCOTT"),
        prob_right,
        &name_to_conincidence,
    );
    assert_eq!(last_name_points, 4.699481);

    let city_by_coincidence = (170 + 1) as f32 / (1592 + 2) as f32;
    let city_name_points = delta_one(true, city_by_coincidence, prob_right);
    println!("city: {:.2} points", city_name_points);
    assert_eq!(city_name_points, 1.7215128);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    // println!(
    //     "post: {:.2} points, {:.5} probability",
    //     post_points,
    //     prob(post_points)
    // );
    assert_eq!(post_points, -0.5820608);

    // Don't see "Bellevue"
    let city_name_points = delta_one(false, city_by_coincidence, prob_right);
    // println!("city: {:.2} points", city_name_points);
    assert_eq!(city_name_points, -0.80281156);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    // println!(
    //     "post: {:.2} points, {:.5} probability",
    //     post_points,
    //     prob(post_points)
    // );
    assert_eq!(post_points, -3.1063852);
}

#[wasm_bindgen_test]
fn test2() {
    struct Person {
        first_name: String,
        last_name: String,
        city: String,
    }

    let prob_member_in_race = 0.01;
    let result_count = 1592;
    let prior_prob = prob_member_in_race / result_count as f32;

    let prob_right = 0.60f32;
    let name_to_conincidence = TokenToCoincidence::default_names();

    // Give a line of race results and a member record, return a probability.
    let result_line = "Scott, Robert, M, Bellevue, 32, 21:00, 1, 10, 5, 100";
    let result_line = result_line.to_uppercase();
    let person = Person {
        first_name: String::from("ROBERT"),
        last_name: String::from("SCOTT"),
        city: String::from("BELLEVUE"),
    };

    let contains_first = result_line.contains(&person.first_name);
    let contains_last = result_line.contains(&person.last_name);
    let contains_city = result_line.contains(&person.city);

    let prior_points = log_odds(prior_prob);

    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let first_name_points = delta_one_name(
        contains_first,
        &Token::new(&person.first_name),
        prob_right,
        &name_to_conincidence,
    );

    // println!("first_name: {:.2} points", first_name_points);
    assert_eq!(first_name_points, 2.9491668);

    let last_name_points = delta_one_name(
        contains_last,
        &Token::new(&person.last_name),
        prob_right,
        &name_to_conincidence,
    );

    // println!("last_name: {:.2} points", last_name_points);
    assert_eq!(last_name_points, 4.699481);

    let city_by_coincidence = (170 + 1) as f32 / (result_count + 2) as f32;
    let city_name_points = delta_one(contains_city, city_by_coincidence, prob_right);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    assert_eq!(post_points, -2.60775);
    // println!(
    //     "post: {:.2} points, {:.5} probability",
    //     post_points,
    //     prob(post_points)
    // );
}
