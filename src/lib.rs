#![allow(clippy::print_literal)]

use std::collections::HashMap;
use std::f32::consts::E;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn delta_one_name(
    contains: bool,
    name: &str,
    prob_right: f32,
    name_to_prob: &NameToProb,
) -> f32 {
    // cmk what if not found?
    let prob_coincidence = name_to_prob.prob(name);
    delta_one(contains, prob_coincidence, prob_right)
}

pub fn delta_many_names(
    contains_list: &[bool],
    name_list: &[&str],
    prob_right_list: &[f32],
    name_to_prob: &NameToProb,
) -> f32 {
    // cmk what if not found?
    // cmk why bother with collect?
    let prob_coincidence_list: Vec<_> = name_list
        .iter()
        .map(|name| name_to_prob.prob(name))
        .collect();
    delta_many(contains_list, &prob_coincidence_list, prob_right_list)
}

fn delta_many(
    contains_list: &[bool],
    prob_coincidence_list: &[f32],
    prob_right_list: &[f32],
) -> f32 {
    assert_eq!(
        contains_list.len(),
        prob_coincidence_list.len(),
        "lengths must match"
    );
    assert_eq!(
        contains_list.len(),
        prob_right_list.len(),
        "lengths must match"
    );
    let zipped = contains_list
        .iter()
        .zip(prob_coincidence_list.iter())
        .zip(prob_right_list.iter());
    zipped
        .map(|((contains, prob_coincidence), prob_right)| {
            delta_one(*contains, *prob_coincidence, *prob_right)
        })
        .reduce(|a, b| a.max(b))
        .unwrap_or_else(|| panic!("Expect length > 0"))
}

#[inline]
pub fn delta_one(contains: bool, prob_coincidence: f32, prob_right: f32) -> f32 {
    if contains {
        (prob_right / prob_coincidence).ln()
    } else {
        ((1.0 - prob_right) / (1.0 - prob_coincidence)).ln()
    }
}

pub fn log_odds(prob: f32) -> f32 {
    prob.ln() - (1.0 - prob).ln()
}

pub fn prob(logodds: f32) -> f32 {
    1.0 / (1.0 + E.powf(-logodds))
}

pub struct NameToProb {
    name_to_prob: HashMap<String, f32>,
    min_prob: f32,
}

impl Default for NameToProb {
    fn default() -> Self {
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
        let min_prob = *name_to_prob
            .values()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        Self {
            name_to_prob,
            min_prob,
        }
    }
}
impl NameToProb {
    pub fn prob(&self, name: &str) -> f32 {
        *self.name_to_prob.get(name).unwrap_or(&self.min_prob)
    }
}

#[test]
fn notebook() {
    // Random line is about Robert

    let prob_member_in_race = 0.01;
    let result_count = 1000;

    let prior_prob = prob_member_in_race / result_count as f32;
    println!("{prior_prob:.5}"); // => 0.00001

    // Our Robert leads to "Robert"

    let prob_right = 0.6;
    println!("{prob_right:.5}");

    // Someone else leads to "Robert"

    let name_to_prob = NameToProb::default();
    let prob_coincidence = name_to_prob.prob("ROBERT");
    println!("{prob_coincidence}"); // => 0.03143

    // "Robert" is from Robert

    let prior_points = log_odds(prior_prob);
    println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");
    assert_eq!(prior_points, -11.512915);

    let delta_points = (prob_right / prob_coincidence).ln();
    assert_eq!(delta_points, 2.9491668);

    let post_points = prior_points + delta_points;
    println!(
        "post: {post_points:.2} points, {:.5} probability",
        prob(post_points)
    );
    assert_eq!(post_points, -8.563747);

    // No "Robert", but still from Robert

    println!("prior: {prior_points:.2} points, {prior_prob:.5} probability");

    let delta_points = ((1.0 - prob_right) / (1.0 - prob_coincidence)).ln();
    assert_eq!(delta_points, -0.88435626);

    let post_points = prior_points + delta_points;
    assert_eq!(post_points, -12.397271);

    // "Robert" and "Scott" is from Robert Scott.

    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let first_name_points = (prob_right / name_to_prob.prob("ROBERT")).ln();
    println!("first_name: {:.2} points", first_name_points);

    let last_name_points = (prob_right / name_to_prob.prob("SCOTT")).ln();
    println!("last_name: {:.2} points", last_name_points);

    let post_points = prior_points + first_name_points + last_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    );
    assert_eq!(post_points, -3.8642664);

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
        let first_name_points =
            delta_one_name(*contains_first_name, first_name, prob_right, &name_to_prob);
        for contains_last_name in [false, true].iter() {
            let last_name_points =
                delta_one_name(*contains_last_name, last_name, prob_right, &name_to_prob);
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
            match (contains_first_name, contains_last_name) {
                (false, false) => assert_eq!(post_points, -13.345492),
                (false, true) => assert_eq!(post_points, -0.40545368),
                (true, false) => assert_eq!(post_points, 0.9389984),
                (true, true) => assert_eq!(post_points, 13.879037),
            }
        }
    }

    // "Bob" is from Robert

    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let mut first_name_points: f32 = f32::NEG_INFINITY;
    for (name, prob_right, contains) in [
        ("ROBERT", 0.50, true),
        ("BOB", 0.05, true),
        ("ROB", 0.05, false),
    ]
    .iter()
    {
        let some_first_name_points = delta_one_name(*contains, name, *prob_right, &name_to_prob);
        println!("\t{}: {:.2} points", name, some_first_name_points);
        first_name_points = first_name_points.max(some_first_name_points);
    }
    println!("first_name: {:.2} points", first_name_points);
    assert_eq!(first_name_points, 4.50986);

    let last_name_points = delta_one_name(true, "SCOTT", prob_right, &name_to_prob);
    println!("last_name: {:.2} points", last_name_points);
    assert_eq!(last_name_points, 4.699481);

    let post_points = prior_points + first_name_points + last_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    );
    assert_eq!(post_points, -2.3035736);

    // "Bellevue" refers to Robert Scott's town.
    println!(
        "prior: {:.2} points, {:.5} probability",
        prior_points, prior_prob
    );

    let first_name_points = delta_many_names(
        &[true, true, false],
        &["ROBERT", "BOB", "ROB"],
        &[0.50, 0.05, 0.05],
        &name_to_prob,
    );
    assert_eq!(first_name_points, 4.50986);

    let last_name_points = delta_one_name(true, "SCOTT", prob_right, &name_to_prob);
    assert_eq!(last_name_points, 4.699481);

    let city_by_coincidence = (170 + 1) as f32 / (1592 + 2) as f32;
    let city_name_points = delta_one(true, city_by_coincidence, prob_right);
    println!("city: {:.2} points", city_name_points);
    assert_eq!(city_name_points, 1.7215128);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    );
    assert_eq!(post_points, -0.5820608);

    // Don't see "Bellevue"
    let city_name_points = delta_one(false, city_by_coincidence, prob_right);
    println!("city: {:.2} points", city_name_points);
    assert_eq!(city_name_points, -0.80281156);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    );
    assert_eq!(post_points, -3.1063852);
}

#[test]
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
    let name_to_prob = NameToProb::default();

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

    // cmk ignoring nicknames for now
    let first_name_points = delta_one_name(
        contains_first,
        &person.first_name,
        prob_right,
        &name_to_prob,
    );

    println!("first_name: {:.2} points", first_name_points);

    let last_name_points =
        delta_one_name(contains_last, &person.last_name, prob_right, &name_to_prob);

    println!("last_name: {:.2} points", last_name_points);

    let city_by_coincidence = (170 + 1) as f32 / (result_count + 2) as f32;
    let city_name_points = delta_one(contains_city, city_by_coincidence, prob_right);

    let post_points = prior_points + first_name_points + last_name_points + city_name_points;
    println!(
        "post: {:.2} points, {:.5} probability",
        post_points,
        prob(post_points)
    );
}
