#![allow(clippy::print_literal)]

use std::collections::HashMap;
use std::f32::consts::E;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn delta(
    name: &str,
    contains: bool,
    name_to_prob: &HashMap<String, f32>,
    prob_name_right: f32,
) -> f32 {
    let prob_coincidence = name_to_prob[name];
    delta_from_coincidence(contains, prob_name_right, prob_coincidence)
}

fn delta_from_coincidence(contains: bool, prob_name_right: f32, prob_coincidence: f32) -> f32 {
    if contains {
        (prob_name_right / prob_coincidence).ln()
    } else {
        ((1.0 - prob_name_right) / (1.0 - prob_coincidence)).ln()
    }
}

fn log_odds(prob: f32) -> f32 {
    prob.ln() - (1.0 - prob).ln()
}

fn prob(logodds: f32) -> f32 {
    1.0 / (1.0 + E.powf(-logodds))
}

fn load_name_to_prob() -> HashMap<String, f32> {
    let name_prob_file =
        File::open(r"C:\Users\carlk\OneDrive\Shares\RaceResults\name_probability.tsv").unwrap();
    let reader = BufReader::new(name_prob_file);
    let mut name_to_prob = HashMap::new();
    for line in reader.lines().skip(1) {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split('\t').collect();
        let name = parts[0].to_string();
        let prob = parts[1].parse::<f32>().unwrap();
        name_to_prob.insert(name, prob);
    }
    name_to_prob
}

#[test]
fn notebook() {
    // Random line is about Robert

    let prob_member_in_race = 0.01;
    let result_count = 1000;

    let prior_prob = prob_member_in_race / result_count as f32;
    println!("{prior_prob:.5}"); // => 0.00001

    // Our Robert leads to "Robert"

    let prob_name_right = 0.6;
    println!("{prob_name_right:.5}");

    // Someone else leads to "Robert"

    let name_to_prob = load_name_to_prob();
    let prob_coincidence = name_to_prob.get("ROBERT").unwrap();
    println!("{prob_coincidence}"); // => 0.03143

    // "Robert" is from Robert

    let prior_points = log_odds(prior_prob);
    println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");

    let delta_points = (prob_name_right / prob_coincidence).ln();
    println!("delta: {delta_points:.2} points");

    let post_points = prior_points + delta_points;
    println!(
        "post: {post_points:.2} points, {:.5} probability",
        prob(post_points)
    ); // => post: -8.56 points, 0.00019 probability

    // No "Robert", but still from Robert

    println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");

    let delta_points = ((1.0 - prob_name_right) / (1.0 - prob_coincidence)).ln();
    println!("delta: {delta_points:.2} points");

    let post_points = prior_points + delta_points;
    println!(
        "post: {post_points:.2} points, {:.6} probability",
        prob(post_points)
    ); // => post: -12.40 points, 0.000004 probability

    // "Robert" and "Scott" is from Robert Scott.

    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let first_name_points = (prob_name_right / name_to_prob["ROBERT"]).ln();
    println!("first_name: {:.2} points", first_name_points);

    let last_name_points = (prob_name_right / name_to_prob["SCOTT"]).ln();
    println!("last_name: {:.2} points", last_name_points);

    let post_points = prior_points + first_name_points + last_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    ); // => post: -3.86 points, 0.02055 probability

    let first_name = "CHELLIE";
    let last_name = "PINGREE";

    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        format!("contains_{first_name}"),
        format!("contains_{last_name}"),
        "prior prob",
        "prior points",
        "first name points",
        "last name points",
        "post points",
        "post prob"
    );

    for contains_first_name in [false, true].iter() {
        let first_name_points = delta(
            first_name,
            *contains_first_name,
            &name_to_prob,
            prob_name_right,
        );
        for contains_last_name in [false, true].iter() {
            let last_name_points = delta(
                last_name,
                *contains_last_name,
                &name_to_prob,
                prob_name_right,
            );
            let post_points = prior_points + first_name_points + last_name_points;

            println!(
                "{}\t{}\t{:.6}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.6}",
                contains_first_name,
                contains_last_name,
                prior_prob,
                prior_points,
                first_name_points,
                last_name_points,
                post_points,
                prob(post_points),
            );
            // => neither name: .0000002, last name: .40, first name: .72, both names: .99
        }
    }

    // "Bob" is from Robert

    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let mut first_name_points: f32 = f32::NEG_INFINITY;
    for (name, prob_name_right, contains) in [
        ("ROBERT", 0.50, true),
        ("BOB", 0.05, true),
        ("ROB", 0.05, false),
    ]
    .iter()
    {
        let some_first_name_points = delta(name, *contains, &name_to_prob, *prob_name_right);
        println!("\t{}: {:.2} points", name, some_first_name_points);
        first_name_points = first_name_points.max(some_first_name_points);
    }
    println!("first_name: {:.2} points", first_name_points);

    let last_name_points = delta("SCOTT", true, &name_to_prob, prob_name_right);
    println!("last_name: {:.2} points", last_name_points);

    let post_points = prior_points + first_name_points + last_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    ); // => post: -2.30 points, 0.09083 probability

    // "Bellevue" refers to Robert Scott's town.
    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let mut first_name_points = f32::NEG_INFINITY;
    for (name, prob_name_right, contains) in [
        ("ROBERT", 0.50, true),
        ("BOB", 0.05, true),
        ("ROB", 0.05, false),
    ]
    .iter()
    {
        let some_first_name_points = delta(name, *contains, &name_to_prob, *prob_name_right);
        println!("\t{}: {:.2} points", name, some_first_name_points);
        first_name_points = first_name_points.max(some_first_name_points);
    }
    println!("first_name: {:.2} points", first_name_points);

    let last_name_points = delta("SCOTT", true, &name_to_prob, prob_name_right);
    println!("last_name: {:.2} points", last_name_points);

    let city_by_coincidence = (170 + 1) as f32 / (1592 + 2) as f32;
    let city_name_points = delta_from_coincidence(true, prob_name_right, city_by_coincidence);
    println!("city: {:.2} points", city_name_points);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    ); // => post: -0.58 points, 0.35846 probability

    // Don't see "Bellevue"
    let city_name_points = delta_from_coincidence(false, prob_name_right, city_by_coincidence);
    println!("city: {:.2} points", city_name_points);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    ); // => post: -0.58 points, 0.35846 probability
}

struct Person {
    first_name: String,
    last_name: String,
    city: String,
}

fn main() {
    let prob_name_right = 0.60f32;
    let name_to_prob = load_name_to_prob();

    // Give a line of race results and a member record, return a probability.
    let result_line = "Scott, Robert, M, Redmond, 32, 21:00, 1, 10, 5, 100";
    let result_line = result_line.to_uppercase();
    let person = Person {
        first_name: String::from("ROBERT"),
        last_name: String::from("SCOTT"),
        city: String::from("BELLEVUE"),
    };

    let contains_first = result_line.contains(&person.first_name);
    let contains_last = result_line.contains(&person.last_name);
    let contains_city = result_line.contains(&person.city);

    // cmk ignoring nicknames for now
    let first_name_points = delta(
        &person.first_name,
        contains_first,
        &name_to_prob,
        prob_name_right,
    );

    let last_name_points = delta(
        &person.last_name,
        contains_last,
        &name_to_prob,
        prob_name_right,
    );
}
