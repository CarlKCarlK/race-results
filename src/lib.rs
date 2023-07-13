// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::print_literal)]
use include_flate::flate;

mod tests;

extern crate alloc;

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

// // cmk file is not local
flate!(static NAME_TO_PROB_STR: str from "../../../Shares/RaceResults/name_probability.tsv");
flate!(static NICKNAMES_STR: str from "examples/nicknames.txt");
flate!(pub static SAMPLE_MEMBERS_STR: str from "../../../Shares/RaceResults/sample_members.no_nicknames.tsv");
flate!(pub static SAMPLE_RESULTS_STR: str from "../../../Shares/RaceResults/sample_results_withcity.txt");

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Token(String);

impl Token {
    pub fn new(s: &str) -> Self {
        // cmk00 regex: check that legal
        Self(s.to_owned())
    }

    pub fn new_or_none(s: &str) -> Option<Self> {
        if !s.is_empty() && !s.chars().any(|c| c.is_ascii_digit()) {
            None
        } else {
            Some(Self::new(s))
        }
    }

    pub fn cmk_clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn delta_one_name(
    contains: bool,
    name: &Token,
    prob_right: f32,
    name_to_coincidence: &TokenToCoincidence,
) -> f32 {
    let prob_coincidence = name_to_coincidence.prob(name);
    delta_one(contains, prob_coincidence, prob_right)
}

pub fn delta_many_names(
    contains_list: &[bool],
    name_list: &[&Token],
    prob_right_list: &[f32],
    name_to_coincidence: &TokenToCoincidence,
) -> f32 {
    // cmk what if not found?
    // cmk why bother with collect?
    let prob_coincidence_list: Vec<_> = name_list
        .iter()
        .map(|name| name_to_coincidence.prob(name))
        .collect();
    delta_many(contains_list, &prob_coincidence_list, prob_right_list)
}

// zero length returns 0.0
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
        .fold(0.0, |a, b| a.max(b))
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

pub struct TokenToCoincidence {
    token_to_prob: HashMap<Token, f32>,
    default: f32,
}

impl TokenToCoincidence {
    pub fn default_names() -> Self {
        let mut name_to_coincidence = HashMap::new();
        // cmk00 regex: We should validate that these names don't have, e.g., * space, ', etc.
        for line in NAME_TO_PROB_STR.lines().skip(1) {
            // cmk what if not two parts?
            let parts: Vec<&str> = line.split('\t').collect();
            // cmk00 regex:: what if not a good token?
            let name = Token::new(parts[0]);
            let prob = parts[1].parse::<f32>().unwrap();
            name_to_coincidence.insert(name, prob);
        }
        let min_prob = *name_to_coincidence
            .values()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        Self {
            token_to_prob: name_to_coincidence,
            default: min_prob,
        }
    }
}
impl TokenToCoincidence {
    pub fn prob(&self, name: &Token) -> f32 {
        *self.token_to_prob.get(name).unwrap_or(&self.default)
    }
}

fn extract_name_to_nicknames_set() -> HashMap<Token, HashSet<Token>> {
    let mut name_to_nickname_set = HashMap::<Token, HashSet<Token>>::new();

    // cmk00 regex: We should valdiate that the names don't have, e.g., * space, ', etc.
    for nickname_line in NICKNAMES_STR.lines() {
        // expect one tab
        let left;
        let right;
        // cmk make this nicer
        if let Some((leftx, rightx)) = nickname_line.split('\t').collect_tuple() {
            // cmk00 regex: nickname data: to upper case. (What's the "ascii" mean here?)
            left = leftx.to_ascii_uppercase();
            right = rightx.to_ascii_uppercase();
        } else {
            panic!("bad nickname line {:?}", nickname_line);
        }

        let left_and_right = [left.clone(), right.clone()];
        let left_and_right = left_and_right
            .iter()
            .map(|side| {
                // cmk00 regex: nickname data: this is the 2nd place we split on /
                side.split('/')
                    .map(|name| {
                        // cmk00 regex: nickname data: We check that ascii or .
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
            let left = Token::new(left);
            for right in left_and_right[1].iter() {
                let right = Token::new(right);
                // cmk00 regex
                name_to_nickname_set
                    .entry(left.cmk_clone())
                    .or_insert_with(HashSet::new)
                    .insert(right.cmk_clone());
                name_to_nickname_set
                    .entry(right)
                    .or_insert_with(HashSet::new)
                    .insert(left.cmk_clone());
            }
        }
    }
    name_to_nickname_set
}

pub struct Config {
    pub prob_member_in_race: f32,
    pub total_right: f32,
    pub total_nickname: f32,
    pub name_to_coincidence: TokenToCoincidence,
    pub stop_words_points: f32,
    pub re: Regex,
    pub threshold_probability: f32,
    pub override_results_count: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prob_member_in_race: 0.01,
            total_right: 0.6,
            total_nickname: 0.1,
            name_to_coincidence: TokenToCoincidence::default_names(),
            stop_words_points: 3.0,
            re: Regex::new(r"[\-/ &\t]+").unwrap(),
            threshold_probability: 0.01,
            override_results_count: None,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    #[anyinput]
    pub fn find_matches(
        &self,
        member_lines: AnyIter<AnyString>,
        result_lines: AnyIter<AnyString>,
        result_lines2: AnyIter<AnyString>,
        include_city: bool,
    ) -> Result<Vec<String>, anyhow::Error> {
        let results_as_tokens = self.tokenize_race_results(result_lines);

        // from the tokens in the race results, look for one's that are too common to be useful
        let (name_stop_words, city_stop_words, city_to_coincidence) =
            self.find_stop_words(&results_as_tokens, include_city);

        let token_to_person_list =
            self.index_person_list(member_lines, name_stop_words, city_stop_words, include_city)?;

        let line_people_list = self.find_matching_people_for_each_result_line(
            result_lines2,
            &results_as_tokens,
            &token_to_person_list,
            &city_to_coincidence,
            include_city,
        );

        let final_output = self.format_final_output(line_people_list);

        Ok(final_output)
    }

    #[anyinput]
    fn tokenize_race_results(&self, result_lines: AnyIter<AnyString>) -> Vec<HashSet<Token>> {
        result_lines
            .map(|result_line| {
                // cmk00 regex: reace results We capitalize the result line before splitting
                let result_line = result_line.as_ref().to_ascii_uppercase();
                // cmk00 regex: We split result lines on the regex
                let token_set: HashSet<Token> = self
                    .re
                    .split(&result_line)
                    .filter_map(Token::new_or_none)
                    // cmk00 regex: With result lines we remove tokens that are empty or that contain any digits.
                    // cmk00 regex: a result token could contain a '*' but it would never match a name or city.
                    .collect();
                // println!("token_set={:?}", token_set);
                token_set
            })
            .collect()
    }

    // cmk "extract_" is a bad name
    fn extract_result_token_and_line_count_list(
        &self,
        results_as_tokens: &[HashSet<Token>],
    ) -> HashMap<Token, usize> {
        let result_token_to_line_count =
            results_as_tokens
                .iter()
                .flatten()
                .fold(HashMap::new(), |mut acc, token| {
                    *acc.entry(token.cmk_clone()).or_insert(0) += 1;
                    acc
                });
        // let mut result_token_to_line_count_vec: Vec<(String, isize)> =
        //     result_token_to_line_count.into_iter().collect();
        // result_token_to_line_count_vec.sort_by_key(|(_token, count)| -*count);
        // result_token_to_line_count_vec
        result_token_to_line_count
    }

    #[allow(clippy::too_many_arguments)]
    #[anyinput]
    fn find_matching_people_for_each_result_line(
        &self,
        result_lines2: AnyIter<AnyString>,
        results_as_tokens: &[HashSet<Token>],
        token_to_person_list: &HashMap<Token, Vec<Rc<Person>>>,
        city_to_coincidence: &TokenToCoincidence,
        include_city: bool,
    ) -> Vec<LinePeople> {
        let results_count = self.extract_results_count(results_as_tokens);
        let prior_points = log_odds(self.prob_member_in_race / results_count as f32);

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

                let name_points_sum = person.name_points(result_tokens, &self.name_to_coincidence);
                let city_points_sum = if include_city {
                    person.city_points(result_tokens, city_to_coincidence)
                } else {
                    0.0
                };

                let post_points = prior_points + name_points_sum + city_points_sum;

                let post_prob = prob(post_points);

                if post_prob > self.threshold_probability {
                    // println!(
                    //     "cmk {person:?} {post_points:.2} {post_prob:.2} {}",
                    //     result_line.as_ref()
                    // );
                    match &mut line_people {
                        None => {
                            line_people = Some(LinePeople {
                                line: result_line.as_ref().to_string(),
                                max_prob: post_prob,
                                person_prob_list: vec![(person.clone(), post_prob)],
                            })
                        }
                        Some(line_people) => {
                            line_people.max_prob = line_people.max_prob.max(post_prob);
                            line_people
                                .person_prob_list
                                .push((person.clone(), post_prob));
                        }
                    };
                }
            }
            if let Some(line_people) = line_people {
                line_people_list.push(line_people);
            }
        }
        line_people_list.sort_by(|a, b| b.max_prob.partial_cmp(&a.max_prob).unwrap());
        line_people_list
    }

    // cmk should tokens be there own type?
    fn extract_dist_list(
        &self,
        name_or_city_phrase: &str,
        token_to_nickname_set: &HashMap<Token, HashSet<Token>>,
    ) -> Result<Vec<Dist>, anyhow::Error> {
        // cmk00 regex: We split name_or_city on whitespace and hyphens, why not the regex?
        name_or_city_phrase
            .split(|c: char| c.is_whitespace() || c == '-')
            .map(|name_or_city| self.split_token(name_or_city, token_to_nickname_set))
            .collect::<Result<Vec<_>, _>>()
    }

    fn split_token(
        &self,
        name_or_city: &str,
        token_to_nickname_set: &HashMap<Token, HashSet<Token>>,
    ) -> Result<Dist, anyhow::Error> {
        // cmk move these
        assert!(
            self.total_nickname <= self.total_right / 2.0,
            "Expect total nickname to be <= than half total_right"
        );
        assert!(
            (0.0..=1.0).contains(&self.total_right),
            "Expect total_right to be between 0 and 1"
        );
        assert!(
            (0.0..=1.0).contains(&self.total_nickname),
            "Expect total_nickname to be between 0 and 1"
        );
        // cmk00 regex: We split the name or city on the regex. How does this fit with extract_dist_list's split?
        let main_set = self
            .re
            .split(name_or_city)
            // cmk00 regex: We filter out empty anything with numbers. Similar code is elsewhere.
            .filter_map(Token::new_or_none)
            .collect::<HashSet<_>>();
        // cmk test that if a nickname is in the main set, it's not in the nickname set
        let nickname_set: HashSet<Token> = main_set
            .iter()
            .filter_map(|token| token_to_nickname_set.get(token))
            .flat_map(|nickname_set| nickname_set.iter())
            .filter(|nickname| !main_set.contains(nickname))
            .map(|nickname| nickname.cmk_clone())
            .collect();

        let mut each_main: f32;
        let mut each_nickname: f32;
        // cmk test each path
        if nickname_set.is_empty() {
            each_main = self.total_right / main_set.len() as f32;
            each_nickname = 0.0;
        } else {
            each_main = (self.total_right - self.total_nickname) / main_set.len() as f32;
            each_nickname = self.total_nickname / nickname_set.len() as f32;
            if each_main < each_nickname {
                each_main = self.total_right / (main_set.len() + nickname_set.len()) as f32;
                each_nickname = each_main;
            }
        }

        let token_sequence = main_set.iter().chain(nickname_set.iter());
        let right_list = repeat(each_main)
            .take(main_set.len())
            .chain(repeat(each_nickname).take(nickname_set.len()));

        let dist = Dist {
            token_and_prob: token_sequence
                .map(|token| token.cmk_clone())
                .zip(right_list)
                .collect_vec(),
        };

        // cmk it doesn't make sense to pull out the strings when we had them earlier
        // cmk assert that every first_name_list, last_name, city contains only A-Z cmk update
        // cmk maybe use compiled regular expressions
        // cmk00 regex
        // cmk00 this is not the right place for this
        // for item in dist.tokens() {
        //     for c in item.chars() {
        //         if !(c.is_ascii_alphabetic() || c == '.' || c == '\'') {
        //             anyhow::bail!("Item '{item}' should contain only A-Za-z, '.', and '\''");
        //         }
        //     }
        // }
        Ok(dist)
    }

    #[allow(clippy::too_many_arguments)]
    #[anyinput]
    fn index_person_list(
        &self,
        member_lines: AnyIter<AnyString>,
        name_stop_words: HashSet<Token>,
        city_stop_words: HashSet<Token>,
        include_city: bool,
    ) -> Result<HashMap<Token, Vec<Rc<Person>>>, anyhow::Error> {
        let city_to_nickname_set = HashMap::<Token, HashSet<Token>>::new();
        let name_to_nickname_set = extract_name_to_nicknames_set();

        let mut token_to_person_list = HashMap::<Token, Vec<Rc<Person>>>::new();
        for (id, line) in member_lines.enumerate() {
            let line = line.as_ref();
            // cmk println!("line={:?}", line);
            let (first_name, last_name, city) = if let Some((first, last, city)) =
                line.split(|c| c == '\t' || c == ',').collect_tuple()
            {
                (first, last, city)
            } else {
                anyhow::bail!(
                    "Line should be First,Last,City separated by tab or comma, not '{line}'"
                );
            };

            // cmk00 regex: We upper case first, last, and city
            let first_name = first_name.to_uppercase();
            let last_name = last_name.to_uppercase();
            let city = city.to_uppercase();

            // cmk00 let's compile first and last sooner.
            let first_dist_list = self.extract_dist_list(&first_name, &name_to_nickname_set)?;
            let last_dist_list = self.extract_dist_list(&last_name, &name_to_nickname_set)?;
            let mut name_dist_list = first_dist_list;
            name_dist_list.extend(last_dist_list);

            // cmk so "Mount/Mt./Mt Si" works, but "NYC/New York City" does not.
            let city_dist_list = self.extract_dist_list(&city, &city_to_nickname_set)?;

            let person = Rc::new(Person {
                name_dist_list,
                city_dist_list,
                id,
            });
            // cmk is there a way to avoid cloning keys?
            // cmk change for loop to use functional
            for name_dist in person.name_dist_list.iter() {
                for name in name_dist.tokens() {
                    if name_stop_words.contains(name) {
                        continue;
                    }
                    token_to_person_list
                        .entry(name.cmk_clone())
                        .or_insert(Vec::new())
                        .push(person.clone());
                }
            }

            if include_city {
                for city_dist in person.city_dist_list.iter() {
                    for city in city_dist.tokens() {
                        if !city_stop_words.contains(city) {
                            token_to_person_list
                                .entry(city.cmk_clone())
                                .or_insert(Vec::new())
                                .push(person.clone());
                        }
                    }
                }
            }
        }
        Ok(token_to_person_list)
    }

    fn extract_results_count(&self, results_as_tokens: &[HashSet<Token>]) -> usize {
        match self.override_results_count {
            Some(results_count) => results_count,
            None => results_as_tokens.len(),
        }
    }

    fn find_stop_words(
        &self,
        results_as_tokens: &[HashSet<Token>],
        include_city: bool,
    ) -> (HashSet<Token>, HashSet<Token>, TokenToCoincidence) {
        let results_count = self.extract_results_count(results_as_tokens);
        let city_coincidence_default = 1f32 / (results_count + 2) as f32;

        let result_token_and_line_count_list =
            self.extract_result_token_and_line_count_list(results_as_tokens);

        let mut name_stop_words = HashSet::<Token>::new();
        let mut city_stop_words = HashSet::<Token>::new();
        let mut city_to_coincidence = TokenToCoincidence {
            token_to_prob: HashMap::new(),
            default: city_coincidence_default,
        };

        for (token, count) in result_token_and_line_count_list.iter() {
            // for each token, in order of decreasing frequency, print its point value as a city and name, present and absent
            if include_city {
                let results_count = self.extract_results_count(results_as_tokens);
                let city_coincidence = (*count + 1) as f32 / (results_count + 2) as f32;
                city_to_coincidence
                    .token_to_prob
                    .insert(token.cmk_clone(), city_coincidence);
                let city_points_contains = delta_one(true, city_coincidence, self.total_right);
                if city_points_contains < self.stop_words_points {
                    city_stop_words.insert(token.cmk_clone());
                }
            }
            let name_points_contains =
                delta_one_name(true, token, self.total_right, &self.name_to_coincidence);
            if name_points_contains < self.stop_words_points {
                name_stop_words.insert(token.cmk_clone());
            }
        }
        (name_stop_words, city_stop_words, city_to_coincidence)
    }

    fn format_final_output(&self, line_people_list: Vec<LinePeople>) -> Vec<String> {
        let mut line_list = Vec::new();
        for line_people in line_people_list.iter() {
            let line = format!("{}", line_people.line);
            line_list.push(line);
            let mut person_prob_list = line_people.person_prob_list.clone();
            // sort by prob
            person_prob_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            for (person, prob) in person_prob_list.iter() {
                let name_list = person
                    .name_dist_list
                    .iter()
                    .map(|name_dist| name_dist.tokens().collect_vec())
                    .collect_vec();
                let city_list = person
                    .city_dist_list
                    .iter()
                    .map(|city_dist| city_dist.tokens().collect_vec())
                    .collect_vec();

                let line = format!(
                    "   {:.2} {:?} {:?}",
                    // cmk if this is useful, make it a method
                    prob,
                    name_list,
                    // cmk if this is useful, make it a method
                    city_list,
                );
                line_list.push(line);
            }
        }
        line_list
    }
}

// cmk0 should O'Neil tokenize to ONEIL?

#[derive(Debug)]
struct Dist {
    token_and_prob: Vec<(Token, f32)>,
}

impl Dist {
    fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.token_and_prob.iter().map(|(token, _prob)| token)
    }

    // cmk return an iterator of f32
    fn probs(&self) -> Vec<f32> {
        self.token_and_prob
            .iter()
            .map(|(_token, prob)| *prob)
            .collect_vec()
    }

    #[allow(clippy::let_and_return)]
    fn delta(&self, contains_list: &[bool], token_to_coincidence: &TokenToCoincidence) -> f32 {
        // cmk what if not found?
        // cmk why bother with collect?
        // it's weird that we look at tokens and probs separately
        let prob_coincidence_list: Vec<_> = self
            .tokens()
            .map(|token| token_to_coincidence.prob(token))
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
    fn points<'a>(
        dist_list: &'a [Dist],
        result_tokens: &'a HashSet<Token>,
        to_coincidence: &'a TokenToCoincidence,
    ) -> impl Iterator<Item = f32> + 'a {
        dist_list.iter().map(move |dist| {
            let contains_list: Vec<_> = dist
                .tokens()
                .map(|token| result_tokens.contains(token))
                .collect();
            // println!("cmk dist {dist:?}, dist.tokens {:?}, result tokens {result_tokens:?} contains_list {contains_list:?}", dist.tokens() );
            // let token = dist.tokens()[0].clone();
            // let contains1 = result_tokens.contains(&token);
            // println!("cmk contains1 {contains1:?}");
            dist.delta(&contains_list, to_coincidence)
        })
    }

    pub fn name_points(
        &self,
        result_tokens: &HashSet<Token>,
        name_to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        Person::points(&self.name_dist_list, result_tokens, name_to_coincidence).sum()
    }

    pub fn city_points(
        &self,
        result_tokens: &HashSet<Token>,
        city_to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        Person::points(&self.city_dist_list, result_tokens, city_to_coincidence)
            .reduce(|a, b| a.max(b))
            .unwrap() // cmk we assume always at least one city
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
// cmk have a page that shows for format of the members file.
// cmk load the page with samples (which means having a small member's input)
// cmk use HTML to show the output nicer
// cmk display every error possible in the input data.
// cmk need to remove ' from names (maybe ".")
// cmk create good sample data
// cmk give users sliders for prob threshold? and priors? etc.
// cmk see the work doc for a mock up of the output
// cmk link: https://carlkcarlk.github.io/race-results/matcher/v0.1.0/index.html
// cmk we use name_to_coincidence twice, but we could use it once.
// cmk will every ESR member be listed when looking at the NYC marathon because 'redmond', etc is rare in the results?
// cmk there must be a way to handle the city/vs not automatically.
// cmk create better sample data
// cmk hard to use from a phone (is there an easy way to access address list?)
// cmk0 understand the regex and similar code. Should it be compiled?

// cmkdoc best on laptop
// cmkdoc many races let see all the results on one page, but some such as the NYC Marathon (with 47,000 runners) doesn't.
// cmkdoc the program assumes one result per line. Sometime when you cut and paste, a result will be split across many lines. I may
// cmkdoc able to fix this in the future for popular websites with a little more code. Please send me examples of race results are split across many lines.
// cmkdoc Mt./Mount/Mt Si but not NYC/New York City -- it splits on hyphens and spaces. Then slashes give alternatives for the single word.
