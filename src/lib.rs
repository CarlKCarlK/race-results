// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::print_literal)]
use include_flate::flate;

mod tests;

extern crate alloc;

use anyhow::anyhow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use alloc::{rc::Rc, string::String, string::ToString, vec::Vec};
use anyinput::anyinput;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use core::{f32::consts::E, iter::repeat};
use itertools::Itertools;
use regex::Regex;

pub fn delta_one_name(
    contains: bool,
    name: &str,
    prob_right: f32,
    name_to_conincidence: &TokenToCoincidence,
) -> f32 {
    // cmk what if not found?
    let prob_coincidence = name_to_conincidence.prob(name);
    delta_one(contains, prob_coincidence, prob_right)
}

#[anyinput]
pub fn delta_many_names(
    contains_list: &[bool],
    name_list: AnyArray<AnyString>,
    prob_right_list: &[f32],
    name_to_conincidence: &TokenToCoincidence,
) -> f32 {
    // cmk what if not found?
    // cmk why bother with collect?
    let prob_coincidence_list: Vec<_> = name_list
        .iter()
        .map(|name| name_to_conincidence.prob(name.as_ref()))
        .collect();
    delta_many(contains_list, &prob_coincidence_list, prob_right_list)
}

pub fn delta_many(
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

// cmk the fields should be private
// cmk should we use 3rd party HashMap?
pub struct TokenToCoincidence {
    pub token_to_prob: HashMap<String, f32>,
    pub default: f32,
}

// // cmk file is not local
flate!(static NAME_TO_PROB_STR: str from "../../../Shares/RaceResults/name_probability.tsv");
flate!(static NICKNAMES_STR: str from "examples/nicknames.txt");
flate!(pub static SAMPLE_MEMBERS_STR: str from "../../../Shares/RaceResults/sample_members.no_nicknames.tsv");
flate!(pub static SAMPLE_RESULTS_STR: str from "../../../Shares/RaceResults/sample_results_withcity.txt");

impl TokenToCoincidence {
    pub fn default_names() -> Self {
        let mut name_to_conincidence = HashMap::new();
        for line in NAME_TO_PROB_STR.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            let name = parts[0];
            let prob = parts[1].parse::<f32>().unwrap();
            name_to_conincidence.insert(name.to_string(), prob);
        }
        let min_prob = *name_to_conincidence
            .values()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        Self {
            token_to_prob: name_to_conincidence,
            default: min_prob,
        }
    }
}
impl TokenToCoincidence {
    pub fn prob(&self, name: &str) -> f32 {
        *self.token_to_prob.get(name).unwrap_or(&self.default)
    }
}

#[anyinput]
pub fn find_matches(
    member_lines: AnyIter<AnyString>,
    result_lines: AnyIter<AnyString>,
    result_lines2: AnyIter<AnyString>,
    include_city: bool,
) -> Result<Vec<String>, anyhow::Error> {
    let prob_member_in_race = 0.01;
    let total_right = 0.6f32;
    let total_nickname = 0.1f32;
    let name_to_coincidence = TokenToCoincidence::default_names();
    let stop_words_points = 3.0f32;
    let threshold_probability = 0.01f32;
    let re = Regex::new(r"[\-/ &\t]+").unwrap();
    let mut name_to_nickname_set = HashMap::<String, HashSet<String>>::new();
    let city_to_nickname_set = HashMap::<String, HashSet<String>>::new();
    for nickname_line in NICKNAMES_STR.lines() {
        // expect one tab
        let left;
        let right;
        // cmk make this nicers
        if let Some((leftx, rightx)) = nickname_line.split('\t').collect_tuple() {
            left = leftx.to_ascii_uppercase();
            right = rightx.to_ascii_uppercase();
        } else {
            panic!("bad nickname line {:?}", nickname_line);
        }

        let left_and_right = [left.clone(), right.clone()];
        let left_and_right = left_and_right
            .iter()
            .map(|side| {
                side.split('/')
                    .map(|name| {
                        // panic if not [A-Z.]
                        name.chars().for_each(|c| {
                            if !c.is_ascii_alphabetic() && c != '.' {
                                panic!("bad char in {:?}", name);
                            }
                        });
                        name
                    })
                    .collect_vec()
            })
            .collect_vec();
        for left in left_and_right[0].iter() {
            for right in left_and_right[1].iter() {
                name_to_nickname_set
                    .entry(left.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(right.to_string());
                name_to_nickname_set
                    .entry(right.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(left.to_string());
            }
        }
    }
    let results_as_tokens: Vec<HashSet<String>> = result_lines
        .map(|result_line| {
            let result_line = result_line.as_ref().to_ascii_uppercase();
            let token_set: HashSet<String> = re
                .split(&result_line)
                .map(|s| s.to_owned())
                .filter(|token| !token.is_empty() && !token.chars().any(|c| c.is_ascii_digit()))
                .collect();
            // println!("token_set={:?}", token_set);
            token_set
        })
        .collect();
    let result_count = results_as_tokens.len();
    let prior_prob = prob_member_in_race / result_count as f32;
    let prior_points = log_odds(prior_prob);
    let city_coincidence_default = 1f32 / (result_count + 2) as f32;
    let result_token_to_line_count =
        results_as_tokens
            .iter()
            .flatten()
            .fold(HashMap::new(), |mut acc, token| {
                *acc.entry(token.clone()).or_insert(0) += 1;
                acc
            });
    let mut result_token_to_line_count_vec: Vec<_> = result_token_to_line_count.iter().collect();
    result_token_to_line_count_vec.sort_by_key(|(_token, count)| -**count);
    let mut city_stop_words = HashSet::<String>::new();
    let mut name_stop_words = HashSet::<String>::new();
    let mut city_to_coincidence = TokenToCoincidence {
        token_to_prob: HashMap::new(),
        default: city_coincidence_default,
    };
    for (token, count) in result_token_to_line_count_vec.iter() {
        // for each token, in order of decreasing frequency, print its point value as a city and name, present and absent
        if include_city {
            let city_coincidence = (*count + 1) as f32 / (result_count + 2) as f32;
            city_to_coincidence
                .token_to_prob
                .insert(token.to_string(), city_coincidence);
            let city_points_contains = delta_one(true, city_coincidence, total_right);
            if city_points_contains < stop_words_points {
                city_stop_words.insert(token.to_string());
            }
        }
        let name_points_contains = delta_one_name(true, token, total_right, &name_to_coincidence);
        // let city_points_absent = delta_one(false, city_coincidence, total_right);
        // let name_points_absent = delta_one_name(false, token, total_right, &name_to_coincidence);
        // println!("{token}\t{count}\t{city_points_contains:.2}\t{city_points_absent:.2}\t{name_points_contains:.2}\t{name_points_absent:.2}");
        if name_points_contains < stop_words_points {
            name_stop_words.insert(token.to_string());
        }
    }
    let mut token_to_person_list: HashMap<String, Vec<Rc<Person>>> = HashMap::new();
    for (id, line) in member_lines.enumerate() {
        let line = line.as_ref();
        // cmk treat first and last more uniformly
        // cmk show a nice error if the line is not tab-separated, three columns
        // cmk println!("line={:?}", line);
        let (first_name, last_name, city) =
            if let Some((first, last, city)) = line.split('\t').collect_tuple() {
                (first, last, city)
            } else {
                return Err(anyhow!(
                    "Line should contain three tab-separated items, not '{line}'"
                ));
            };

        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();

        let first_dist = Dist::split_token(
            &first_name,
            total_right,
            &re,
            &name_to_nickname_set,
            total_nickname,
        )?;
        let last_dist = Dist::split_token(
            &last_name,
            total_right,
            &re,
            &name_to_nickname_set,
            total_nickname,
        )?;
        let name_dist_list = vec![first_dist, last_dist];

        // cmk so "Mount/Mt./Mt Si" works, but "NYC/New York City" does not.
        let city_dist_list = city
            .split_ascii_whitespace()
            .map(|city| {
                Dist::split_token(
                    city,
                    total_right,
                    &re,
                    &city_to_nickname_set,
                    total_nickname,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let person = Rc::new(Person {
            name_dist_list,
            city_dist_list,
            id,
        });
        // cmk is there a way to avoid cloning keys?
        // cmk change for loop to use functional
        for name_dist in person.name_dist_list.iter() {
            for name in name_dist.tokens().iter() {
                if name_stop_words.contains(name) {
                    continue;
                }
                token_to_person_list
                    .entry(first_name.clone())
                    .or_insert(Vec::new())
                    .push(person.clone());
            }
        }

        if include_city {
            for city_dist in person.city_dist_list.iter() {
                for city in city_dist.tokens().iter() {
                    if !city_stop_words.contains(city) {
                        token_to_person_list
                            .entry(city.clone())
                            .or_insert(Vec::new())
                            .push(person.clone());
                    }
                }
            }
        }
    }
    let mut line_people_list: Vec<LinePeople> = Vec::new();
    for (result_line, result_tokens) in result_lines2.zip(results_as_tokens)
    // .take(100)
    {
        // let cmk = result_line.as_ref().clone();
        // if cmk.contains("test") {
        //     println!("result_line");
        // }
        let person_set = result_tokens
            .iter()
            .filter_map(|token| token_to_person_list.get(token))
            .flatten()
            .collect::<HashSet<_>>();

        let mut line_people: Option<LinePeople> = None;
        for person in person_set.iter() {
            let person = *person;

            let name_points_sum = person.name_points(&result_tokens, &name_to_coincidence);
            let city_points_sum = if include_city {
                person.city_points(&result_tokens, &city_to_coincidence)
            } else {
                0.0
            };

            let post_points = prior_points + name_points_sum + city_points_sum;

            let post_prob = prob(post_points);

            if post_prob > threshold_probability {
                // println!(
                //     "cmk {person:?} {post_points:.2} {post_prob:.2} {}",
                //     result_line.as_ref()
                // );
                if let Some(line_people) = &mut line_people {
                    line_people.max_prob = line_people.max_prob.max(post_prob);
                    line_people
                        .person_prob_list
                        .push((person.clone(), post_prob));
                } else {
                    line_people = Some(LinePeople {
                        line: result_line.as_ref().to_string(),
                        max_prob: post_prob,
                        person_prob_list: vec![(person.clone(), post_prob)],
                    });
                }
            }
        }
        if let Some(line_people) = line_people {
            line_people_list.push(line_people);
        }
    }
    let mut line_list = Vec::new();
    line_people_list.sort_by(|a, b| b.max_prob.partial_cmp(&a.max_prob).unwrap());
    for line_people in line_people_list.iter() {
        let line = format!("{}", line_people.line);
        line_list.push(line);
        let mut person_prob_list = line_people.person_prob_list.clone();
        // sort by prob
        person_prob_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (person, prob) in person_prob_list.iter() {
            let line = format!(
                "   {:.2} {:?} {:?}",
                // cmk if this is useful, make it a method
                prob,
                person
                    .name_dist_list
                    .iter()
                    .map(|name_dist| name_dist.tokens())
                    .collect_vec(),
                // cmk if this is useful, make it a method
                person
                    .city_dist_list
                    .iter()
                    .map(|city_dist| city_dist.tokens())
                    .collect_vec(),
            );
            line_list.push(line);
        }
    }
    Ok(line_list)
}

// cmk should O'Neil tokenize to ONEIL?
// cmk be sure there is a way to run without matching on city
// cmk what about people with two-part first names?

#[derive(Debug)]
struct Dist {
    token_and_prob: Vec<(String, f32)>,
}

impl Dist {
    // cmk return an iterator of &str
    fn tokens(&self) -> Vec<String> {
        self.token_and_prob
            .iter()
            .map(|(token, _prob)| token.clone())
            .collect_vec()
    }

    // cmk return an iterator of f32
    fn probs(&self) -> Vec<f32> {
        self.token_and_prob
            .iter()
            .map(|(_token, prob)| *prob)
            .collect_vec()
    }

    // cmk shouldn't have to pass in the re
    fn split_token(
        name: &str,
        total_right: f32,
        re: &Regex,
        token_to_nickname_set: &HashMap<String, HashSet<String>>,
        total_nickname: f32,
    ) -> Result<Self, anyhow::Error> {
        // cmk0
        assert!(
            total_nickname <= total_right / 2.0,
            "Expect total nickname to be <= than half total_right"
        );
        // cmk0
        assert!(
            (0.0..=1.0).contains(&total_right),
            "Expect total_right to be between 0 and 1"
        );
        // cmk0
        assert!(
            (0.0..=1.0).contains(&total_nickname),
            "Expect total_nickname to be between 0 and 1"
        );
        let main_set = re.split(name).map(|s| s.to_owned()).collect::<HashSet<_>>();
        // cmk test that if a nickname is in the main set, it's not in the nickname set
        let nickname_set: HashSet<_> = main_set
            .iter()
            .filter_map(|token| token_to_nickname_set.get(token))
            .flat_map(|nickname_set| nickname_set.iter().cloned())
            .filter(|nickname| !main_set.contains(nickname))
            .collect();

        let mut each_main: f32;
        let mut each_nickname: f32;
        // cmk test each path
        if nickname_set.is_empty() {
            each_main = total_right / main_set.len() as f32;
            each_nickname = 0.0;
        } else {
            each_main = (total_right - total_nickname) / main_set.len() as f32;
            each_nickname = total_nickname / nickname_set.len() as f32;
            if each_main < each_nickname {
                each_main = total_right / (main_set.len() + nickname_set.len()) as f32;
                each_nickname = each_main;
            }
        }

        let token_list = main_set
            .iter()
            .chain(nickname_set.iter())
            .cloned()
            .collect_vec();
        let right_list = repeat(each_main)
            .take(main_set.len())
            .chain(repeat(each_nickname).take(nickname_set.len()))
            .collect_vec();

        let dist = Dist {
            token_and_prob: token_list.into_iter().zip(right_list).collect_vec(),
        };

        // cmk it doesn't make sense to pull out the strings when we had them earlier
        // cmk should return an error rather than panic
        // cmk assert that every first_name_list, last_name, city contains only A-Z cmk update
        // cmk maybe use compiled regular expressions
        for item in dist.tokens().iter() {
            for c in item.chars() {
                if !(c.is_ascii_alphabetic() || c == '.' || c == '\'') {
                    // cmk0 replace my anyhow format in other places with this
                    anyhow::bail!("Item '{item}' should contain only A-Za-z, '.', and '\''");
                }
            }
        }
        Ok(dist)
    }

    #[allow(clippy::let_and_return)]
    fn delta(&self, contains_list: &[bool], token_to_coincidence: &TokenToCoincidence) -> f32 {
        // cmk what if not found?
        // cmk why bother with collect?
        // it's weird that we look at tokens and probs separately
        let prob_coincidence_list: Vec<_> = self
            .tokens()
            .iter()
            .map(|token| token_to_coincidence.prob(token.as_ref()))
            .collect();
        // cmk merge delta_many code to here
        let delta = delta_many(contains_list, &prob_coincidence_list, &self.probs());
        // println!("cmk {self:?} {delta:?}");
        delta
    }
}

// cmk which is it a Person and a Member?
#[derive(Debug)]
struct Person {
    name_dist_list: Vec<Dist>,
    city_dist_list: Vec<Dist>,
    id: usize,
}

impl Person {
    fn points(
        dist_list: &[Dist],
        result_tokens: &HashSet<String>,
        to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        dist_list
            .iter()
            .map(|dist| {
                let contains_list: Vec<_> = dist
                    .tokens()
                    .iter()
                    .map(|token| result_tokens.contains(token))
                    .collect();
                dist.delta(&contains_list, to_coincidence)
            })
            .sum()
    }

    pub fn name_points(
        &self,
        result_tokens: &HashSet<String>,
        name_to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        Person::points(&self.name_dist_list, result_tokens, name_to_coincidence)
    }

    pub fn city_points(
        &self,
        result_tokens: &HashSet<String>,
        city_to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        Person::points(&self.city_dist_list, result_tokens, city_to_coincidence)
    }
}

impl Ord for Person {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Person {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Hash for Person {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for Person {}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

struct LinePeople {
    line: String,
    max_prob: f32,
    person_prob_list: Vec<(Rc<Person>, f32)>,
}

pub fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

// cmk make the results paste in window small
// cmk make the output window as large as needed.
// cmk have a page that shows for format of the members file.
// cmk load the page with samples (which means having a small member's input)
// cmk should the members file have a header?
// cmk use HTML to show the output nicer
// cmk display every error possible in the input data.
// cmk0 need to do multi-part last names and first names (space and -)
// cmk0 need to remove ' from names (maybe ".")
// cmk put sample data in project
// cmk create good sample data
// cmk give users sliders for prob threshold? and priors? etc.
// cmk see the work doc for a mock up of the output
// cmk link: https://carlkcarlk.github.io/race-results/matcher/v0.1.0/index.html
