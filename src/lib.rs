// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::print_literal)]
use include_flate::flate;

mod tests;

extern crate alloc;

use core::fmt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
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

// // cmk file is not local
flate!(static NAME_TO_PROB_STR: str from "../../Shares/RaceResults/name_probability.tsv");
flate!(static NICKNAMES_STR: str from "examples/nicknames.txt");
flate!(pub static SAMPLE_MEMBERS_STR: str from "../../Shares/RaceResults/sample_members.no_nicknames.tsv");
flate!(pub static SAMPLE_RESULTS_STR: str from "../../Shares/RaceResults/sample_results_withcity.txt");

fn is_comma_or_tab(c: char) -> bool {
    c == ',' || c == '\t'
}
fn is_slash_or_ampersand(c: char) -> bool {
    c == '/' || c == '&'
}
fn is_whitespace_or_dash(c: char) -> bool {
    c.is_whitespace() || c == '-'
}
fn is_any_separator(c: char) -> bool {
    is_comma_or_tab(c) || is_slash_or_ampersand(c) || is_whitespace_or_dash(c)
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Token(String);

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Token {
    pub fn new(s: &str) -> Self {
        Self(Token::to_canonical(s).unwrap())
    }

    // A-Za-z . '
    // internally we got to uppercase, remove . and ' and it can't be then empty.
    pub fn to_canonical(s: &str) -> Result<String, anyhow::Error> {
        let s = s.to_uppercase().replace(['.', '\''], "");
        if s.is_empty() || s.chars().any(|c| !c.is_ascii_alphabetic()) {
            Err(anyhow::anyhow!(
                "String must be alphabetic with (ignored . and ') and then not empty, not \"{}\".",
                s
            ))
        } else {
            Ok(s)
        }
    }

    pub fn new_or_error(s: &str) -> Result<Token, anyhow::Error> {
        match Token::to_canonical(s) {
            Ok(s) => Ok(Self(s)),
            Err(e) => Err(e),
        }
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
    contains_list: impl Iterator<Item = bool>,
    name_list: &[&Token],
    prob_right_list: impl Iterator<Item = f32>,
    name_to_coincidence: &TokenToCoincidence,
) -> f32 {
    let prob_coincidence_sequence = name_list.iter().map(|name| name_to_coincidence.prob(name));
    delta_many(contains_list, prob_coincidence_sequence, prob_right_list)
}

fn max_abs(a: f32, b: f32) -> f32 {
    if a.abs() > b.abs() {
        a
    } else {
        b
    }
}

// zero length returns 0.0
// cmk is there a big where it doesn't do negatives correctly?
pub fn delta_many(
    contains_list: impl Iterator<Item = bool>,
    prob_coincidence_sequence: impl Iterator<Item = f32>,
    prob_right_list: impl Iterator<Item = f32>,
) -> f32 {
    let zipped = contains_list
        .zip_eq(prob_coincidence_sequence)
        .zip_eq(prob_right_list);
    zipped
        .map(|((contains, prob_coincidence), prob_right)| {
            delta_one(contains, prob_coincidence, prob_right)
        })
        .fold(0.0f32, max_abs)
}
// cmk is max the right function when combining all negatives?

// cmk #[inline]
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
        for line in NAME_TO_PROB_STR.lines().skip(1) {
            let (name, prob) = line.split(is_comma_or_tab).collect_tuple().unwrap();
            let name = Token::new(name);
            let prob = prob.parse::<f32>().unwrap();
            name_to_coincidence.insert(name, prob);
        }
        let min_prob = name_to_coincidence.values().fold(1.0f32, |a, b| a.min(*b));
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

    for nickname_line in NICKNAMES_STR.lines() {
        let left_and_right: Vec<&str> = nickname_line.split(is_comma_or_tab).collect_vec();
        assert_eq!(
            left_and_right.len(),
            2,
            "Expect two tab-separated parts to nickname line, not {:?}",
            nickname_line
        );
        let left_and_right = left_and_right
            .iter()
            .map(|side| {
                side.split(is_slash_or_ampersand)
                    .map(Token::new_or_error)
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>();

        let left_and_right = left_and_right.unwrap_or_else(|e| {
            panic!(
                "Error parsing nickname line {:?} with error {:?}",
                nickname_line, e
            )
        });

        for left in left_and_right[0].iter() {
            for right in left_and_right[1].iter() {
                name_to_nickname_set
                    .entry(left.clone())
                    .or_insert_with(HashSet::new)
                    .insert(right.clone());
                name_to_nickname_set
                    .entry(right.clone())
                    .or_insert_with(HashSet::new)
                    .insert(left.clone());
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
        self.assert_that_config_is_valid();

        let results_as_tokens = self.tokenize_race_results(result_lines);

        // Look for tokens in the race results that are too common to be useful
        let (name_stop_words, city_stop_words, city_to_coincidence) =
            self.find_stop_words(&results_as_tokens);

        let token_to_person_list =
            self.index_person_list(member_lines, name_stop_words, city_stop_words, include_city)?;

        let line_people_list = self.find_matching_people_for_each_result_line(
            result_lines2,
            &results_as_tokens,
            &token_to_person_list,
            &city_to_coincidence,
        );

        let final_output = self.format_final_output(line_people_list);

        Ok(final_output)
    }

    fn assert_that_config_is_valid(&self) {
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
    }

    #[anyinput]
    fn tokenize_race_results(&self, result_lines: AnyIter<AnyString>) -> Vec<HashSet<Token>> {
        result_lines
            .map(|result_line| {
                let result_line = result_line.as_ref();
                let token_set: HashSet<Token> = result_line
                    .split(is_any_separator)
                    .filter_map(|s| Token::new_or_error(s).ok())
                    .collect();
                // println!("token_set={:?}", token_set);
                token_set
            })
            .collect()
    }

    // cmk "extract_" is a bad name
    fn count_result_tokens(&self, results_as_tokens: &[HashSet<Token>]) -> HashMap<Token, usize> {
        let result_token_to_line_count =
            results_as_tokens
                .iter()
                .flatten()
                .fold(HashMap::new(), |mut acc, token| {
                    *acc.entry(token.clone()).or_insert(0) += 1;
                    acc
                });
        // let mut result_token_to_line_count_vec: Vec<(String, isize)> =
        //     result_token_to_line_count.into_iter().collect();
        // result_token_to_line_count_vec.sort_by_key(|(_token, count)| -*count);
        // result_token_to_line_count_vec
        result_token_to_line_count
    }

    fn annotate_line<T: Score>(
        result_line: &str,
        result_tokens: &HashSet<Token>,
        all_points: &T,
    ) -> String {
        let annotations = all_points.annotations();

        // Find the first location of every token in the result line
        let mut token_to_range: HashMap<Token, Range<usize>> = HashMap::new();
        for s in result_line.split(is_any_separator) {
            if let Ok(token) = Token::new_or_error(s) {
                token_to_range.entry(token).or_insert_with(|| {
                    let offset = s.as_ptr() as usize - result_line.as_ptr() as usize;
                    offset..offset + s.len()
                });
            }
        }

        // Group annotations by common token
        let mut token_to_annotation_list: HashMap<Token, Vec<Annotation>> = HashMap::new();
        for annotation in annotations {
            let get = token_to_annotation_list.get_mut(&annotation.token);
            if let Some(list) = get {
                list.push(annotation);
            } else {
                let list = vec![annotation.clone()];
                token_to_annotation_list.insert(annotation.token.clone(), list);
            }
        }

        // Find the location of every token in the annotations (some will be None)
        let mut token_and_range = token_to_annotation_list
            .keys()
            .map(|token| {
                let range_or_none = token_to_range.get(token).cloned();
                (token.clone(), range_or_none)
            })
            .collect_vec();

        // Sort the locations/tokens from back to found (with None first)
        token_and_range.sort_by(|a, b| {
            let a_range_or_none = a.1.as_ref();
            let b_range_or_none = b.1.as_ref();
            match (a_range_or_none, b_range_or_none) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a_range), Some(b_range)) => b_range.start.cmp(&a_range.start),
            }
        });

        // for every location/token (from the back), insert the annotationS into the result line
        let mut annotated = result_line.to_string();
        for (token, range_or_none) in token_and_range {
            let annotation_set = &token_to_annotation_list[&token];
            let pts_str = annotation_set
                .iter()
                .map(|annotation| format!("{:+.2}", annotation.delta))
                .join("");
            if let Some(range) = range_or_none {
                let s = &result_line[range.clone()];
                annotated.replace_range(
                    range.clone(),
                    &format!("<mark>{s}<sup>{pts_str} pts</sup></mark>"),
                );
            } else {
                annotated.push_str(&format!("<mark style=\"background-color: red; color: white;\"> -Missing: {token}<sup>{pts_str} pts</sup></mark>"));
            }
        }
        annotated
    }

    #[allow(clippy::too_many_arguments)]
    #[anyinput]
    fn find_matching_people_for_each_result_line(
        &self,
        result_lines2: AnyIter<AnyString>,
        results_as_tokens: &[HashSet<Token>],
        token_to_person_list: &HashMap<Token, Vec<Rc<Person>>>,
        city_to_coincidence: &TokenToCoincidence,
    ) -> Vec<LinePeople> {
        let results_count = self.results_count(results_as_tokens);
        let prior_points = log_odds(self.prob_member_in_race / results_count as f32);

        let mut line_people_list: Vec<LinePeople> = Vec::new();

        // for each line in the results
        for (result_line, result_tokens) in result_lines2.zip(results_as_tokens) {
            let result_line = result_line.as_ref();
            // find people with at least one token in common with the result line
            let person_set = result_tokens
                .iter()
                .filter_map(|token| token_to_person_list.get(token))
                .flatten()
                .collect::<HashSet<_>>();

            let mut line_people: Option<LinePeople> = None;
            for person in person_set.iter() {
                let person = *person;

                let name_points = person.name_points(result_tokens, &self.name_to_coincidence);
                let city_points = person.city_points(result_tokens, city_to_coincidence);
                let all_points: Vec<Box<dyn Score>> =
                    vec![Box::new(name_points), Box::new(city_points)];
                let all_points: IndScoreList = all_points.into_iter().collect();

                // let name_points = person.name_points(result_tokens, &self.name_to_coincidence);
                // let city_points = person.city_points(result_tokens, city_to_coincidence);

                let post_points = prior_points + all_points.delta();
                let post_prob = prob(post_points);

                if post_prob > self.threshold_probability {
                    match &mut line_people {
                        None => {
                            let annotated_line =
                                Config::annotate_line(result_line, result_tokens, &all_points);
                            line_people = Some(LinePeople {
                                line: result_line.to_string(),
                                show_work: format!(
                                    "<tr><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>",
                                    result_line,
                                    person,
                                    annotated_line,
                                    all_points.html(),
                                ),
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

    fn extract_dist_list(
        &self,
        name_or_city_phrase: &str,
        token_to_nickname_set: &HashMap<Token, HashSet<Token>>,
    ) -> Result<Vec<Dist>, anyhow::Error> {
        name_or_city_phrase
            .split(is_whitespace_or_dash)
            .map(|name_or_city| self.split_token(name_or_city, token_to_nickname_set))
            .collect::<Result<Vec<_>, _>>()
    }

    fn split_token(
        &self,
        name_or_city: &str,
        token_to_nickname_set: &HashMap<Token, HashSet<Token>>,
    ) -> Result<Dist, anyhow::Error> {
        let main_set = name_or_city
            .split(is_slash_or_ampersand)
            .filter(|name| !name.is_empty())
            .map(Token::new_or_error)
            .collect::<Result<HashSet<_>, _>>()?;

        let nickname_set: HashSet<Token> = main_set
            .iter()
            .filter_map(|token| token_to_nickname_set.get(token))
            .flat_map(|nickname_set| nickname_set.iter())
            .filter(|nickname| !main_set.contains(nickname))
            .cloned()
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
            token_and_prob: token_sequence.cloned().zip(right_list).collect_vec(),
        };

        Ok(dist)
    }

    fn insert_into_map(
        token_to_person_list: &mut HashMap<Token, Vec<Rc<Person>>>,
        token: &Token,
        person: &Rc<Person>,
    ) {
        token_to_person_list
            .entry(token.clone())
            .or_insert(Vec::new())
            .push(person.clone());
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
        let name_to_nickname_set = extract_name_to_nicknames_set();

        let mut token_to_person_list = HashMap::<Token, Vec<Rc<Person>>>::new();
        for (id, line) in member_lines.enumerate() {
            let line = line.as_ref();
            let fields = line.split(is_comma_or_tab).collect_vec();
            if fields.len() != 3 {
                anyhow::bail!(
                    "Line should be First,Last,City separated by tab or comma, not '{line}'"
                );
            }
            let name = format!("{} {}", fields[0], fields[1]);
            let name_dist_list = self.extract_dist_list(&name, &name_to_nickname_set)?;

            let city = if include_city { fields[2] } else { "" };
            let city_to_nickname_set = HashMap::<Token, HashSet<Token>>::new(); // currently empty
            let city_dist_list = self.extract_dist_list(city, &city_to_nickname_set)?;

            let person = Rc::new(Person {
                name_dist_list,
                city_dist_list,
                id,
            });

            person
                .name_dist_list
                .iter()
                .flat_map(|name_dist| name_dist.tokens())
                .filter(|name| !name_stop_words.contains(name))
                .for_each(|name| Self::insert_into_map(&mut token_to_person_list, name, &person));

            person
                .city_dist_list
                .iter()
                .flat_map(|city_dist| city_dist.tokens())
                .filter(|city| !city_stop_words.contains(city))
                .for_each(|city| Self::insert_into_map(&mut token_to_person_list, city, &person));
        }
        Ok(token_to_person_list)
    }

    fn results_count(&self, results_as_tokens: &[HashSet<Token>]) -> usize {
        match self.override_results_count {
            Some(results_count) => results_count,
            None => results_as_tokens.len(),
        }
    }

    fn find_stop_words(
        &self,
        results_as_tokens: &[HashSet<Token>],
    ) -> (HashSet<Token>, HashSet<Token>, TokenToCoincidence) {
        let results_count = self.results_count(results_as_tokens);
        let city_coincidence_default = 1f32 / (results_count + 2) as f32;

        let result_token_and_line_count_list = self.count_result_tokens(results_as_tokens);

        let mut name_stop_words = HashSet::<Token>::new();
        let mut city_stop_words = HashSet::<Token>::new();
        let mut city_to_coincidence = TokenToCoincidence {
            token_to_prob: HashMap::new(),
            default: city_coincidence_default,
        };

        for (token, count) in result_token_and_line_count_list.iter() {
            let results_count = self.results_count(results_as_tokens);
            let city_coincidence = (*count + 1) as f32 / (results_count + 2) as f32;
            city_to_coincidence
                .token_to_prob
                .insert(token.clone(), city_coincidence);
            let city_points_contains = delta_one(true, city_coincidence, self.total_right);
            if city_points_contains < self.stop_words_points {
                city_stop_words.insert(token.clone());
            }
            let name_points_contains =
                delta_one_name(true, token, self.total_right, &self.name_to_coincidence);
            if name_points_contains < self.stop_words_points {
                name_stop_words.insert(token.clone());
            }
        }
        (name_stop_words, city_stop_words, city_to_coincidence)
    }

    // cmk this should be a method of LinePeople ???
    fn format_final_output(&self, line_people_list: Vec<LinePeople>) -> Vec<String> {
        let mut line_list = Vec::new();
        for line_people in line_people_list.iter() {
            line_list.push(line_people.line.to_string());
            line_list.push(line_people.show_work.to_string());
            // let mut person_prob_list = line_people.person_prob_list.clone();
            // // sort by prob
            // person_prob_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            // for (person, prob) in person_prob_list.iter() {
            //     let name_list = person
            //         .name_dist_list
            //         .iter()
            //         .map(|name_dist| name_dist.tokens().collect_vec())
            //         .collect_vec();
            //     let city_list = person
            //         .city_dist_list
            //         .iter()
            //         .map(|city_dist| city_dist.tokens().collect_vec())
            //         .collect_vec();

            //     let line = format!("   {:.2} {:?} {:?}", prob, name_list, city_list,);
            //     line_list.push(line);
            // }
        }
        line_list
    }
}

#[derive(Debug)]
struct Dist {
    token_and_prob: Vec<(Token, f32)>,
}

impl Dist {
    fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.token_and_prob.iter().map(|(token, _prob)| token)
    }

    fn probs(&self) -> impl Iterator<Item = f32> + '_ {
        self.token_and_prob.iter().map(|(_token, prob)| *prob)
    }

    // fn delta(
    //     &self,
    //     contains_list: impl Iterator<Item = bool>,
    //     token_to_coincidence: &TokenToCoincidence,
    // ) -> f32 {
    //     // cmk0
    //     let prob_coincidence_sequence = self.tokens().map(|token| token_to_coincidence.prob(token));
    //     delta_many(contains_list, prob_coincidence_sequence, self.probs())
    // }
}

// cmk which is it a Person and a Member?
#[derive(Debug)]
struct Person {
    name_dist_list: Vec<Dist>,
    city_dist_list: Vec<Dist>,
    id: usize,
}

trait Score: core::fmt::Debug {
    fn delta(&self) -> f32;
    fn html(&self) -> String;
    fn annotations(&self) -> Vec<Annotation>;
}

#[derive(Clone)]
struct Annotation {
    token: Token,
    delta: f32,
}

#[derive(Debug)]
struct SingleScore {
    token: Token,
    contains: bool,
    prob_right: f32,
    prob_coincidence: f32,
    delta: f32,
}

impl SingleScore {
    fn new(
        result_tokens: &HashSet<Token>,
        token: &Token,
        to_coincidence: &TokenToCoincidence,
        prob: &f32,
    ) -> SingleScore {
        let contains = result_tokens.contains(token);
        let prob_coincidence = to_coincidence.prob(token);
        let prob_right = *prob;
        let delta_inner = delta_one(contains, prob_coincidence, prob_right);
        let score = SingleScore {
            token: token.clone(),
            contains,
            prob_right,
            prob_coincidence,
            delta: delta_inner, // cmk don't let api users set this themselves
        };
        score
    }
}

impl Score for SingleScore {
    fn annotations(&self) -> Vec<Annotation> {
        vec![Annotation {
            token: self.token.clone(),
            delta: self.delta,
        }]
    }
    fn delta(&self) -> f32 {
        self.delta
    }

    fn html(&self) -> String {
        format!(
                "<table border=\"1\" style=\"border-collapse: collapse; margin-right: 20px;\">
                    <tr>
                        <th colspan=\"2\" style=\"text-align: center; font-weight: bold;\">SingleScore</th>
                    </tr>
                    <tr>
                        <td>token</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>contains</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>prob_right</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>prob_coincid</td>
                        <td>{}</td>
                    </tr>
                    <tr>
                        <td>delta</td>
                        <td>{}</td>
                    </tr>
                </table>",
                self.token.to_string(),
                self.contains,
                self.prob_right,
                self.prob_coincidence,
                self.delta
            )
    }
}

#[derive(Debug)]
struct DepScoreList {
    score_list: Vec<Box<dyn Score>>,
    delta: f32,
}

impl Score for DepScoreList {
    fn annotations(&self) -> Vec<Annotation> {
        // cmk can/should abs_max be a method?
        let abs_max = self
            .score_list
            .iter()
            .find(|score| score.delta() == self.delta);
        match abs_max {
            Some(abs_max) => abs_max.annotations(),
            None => vec![],
        }
    }

    fn delta(&self) -> f32 {
        self.delta
    }
    fn html(&self) -> String {
        format!(
            "<table border=\"1\" style=\"border-collapse: collapse; width: max-content;\">
        <tr>
            <th colspan=\"2\" style=\"text-align: center; font-weight: bold;\">DepScoreList</th>
        </tr>
        <tr>
            <td>score list</td>
            <td>
                <div style=\"display: flex; align-items: flex-start;\">
                    {score_list}
                </div>
            </td>
        </tr>
        <tr>
            <td>delta</td>
            <td>{delta}</td>
        </tr>
    </table>
    ",
            score_list = self
                .score_list
                .iter()
                .map(|score| score.html())
                .collect::<Vec<String>>()
                .join("\n",),
            delta = self.delta
        )
    }
}

impl Default for DepScoreList {
    fn default() -> Self {
        Self {
            score_list: Vec::new(),
            delta: 0.0,
        }
    }
}

impl FromIterator<Box<dyn Score>> for DepScoreList {
    fn from_iter<T: IntoIterator<Item = Box<dyn Score>>>(iter: T) -> Self {
        let mut dep_score_list = DepScoreList::default();
        for score in iter {
            dep_score_list.push(score);
        }
        dep_score_list
    }
}

impl DepScoreList {
    fn push(&mut self, score: Box<dyn Score>) {
        let score_delta = score.delta();
        if self.score_list.len() == 0 || self.delta.abs() < score_delta.abs() {
            self.delta = score_delta;
        }
        self.score_list.push(score);
    }

    fn new(
        dist: &Dist,
        result_tokens: &HashSet<Token>,
        to_coincidence: &TokenToCoincidence,
    ) -> DepScoreList {
        let mut dep_score_list = DepScoreList::default();
        for (token, prob) in dist.token_and_prob.iter() {
            let score = SingleScore::new(result_tokens, token, to_coincidence, prob);
            dep_score_list.push(Box::new(score));
        }
        dep_score_list
    }
}

// cmk most of the code is the same between IndScoreList and DepScoreList
#[derive(Debug)]
struct IndScoreList {
    score_list: Vec<Box<dyn Score>>,
    delta: f32,
}

impl Score for IndScoreList {
    fn annotations(&self) -> Vec<Annotation> {
        self.score_list
            .iter()
            .flat_map(|score| score.annotations())
            .collect()
    }
    fn delta(&self) -> f32 {
        self.delta
    }
    fn html(&self) -> String {
        format!(
            "<table border=\"1\" style=\"border-collapse: collapse; width: max-content;\">
            <tr>
                <th colspan=\"2\" style=\"text-align: center; font-weight: bold;\">IndScoreList</th>
            </tr>
            <tr>
                <td>score list</td>
                <td>
                    <div style=\"display: flex; align-items: flex-start;\">
                        {score_list}
                    </div>
                </td>
            </tr>
            <tr>
                <td>delta</td>
                <td>{delta}</td>
            </tr>
        </table>
        ",
            score_list = self
                .score_list
                .iter()
                .map(|score| score.html())
                .collect::<Vec<String>>()
                .join("\n",),
            delta = self.delta
        )
    }
}

impl Default for IndScoreList {
    fn default() -> Self {
        Self {
            score_list: Vec::new(),
            delta: 0.0,
        }
    }
}

impl FromIterator<Box<dyn Score>> for IndScoreList {
    fn from_iter<T: IntoIterator<Item = Box<dyn Score>>>(iter: T) -> Self {
        let mut ind_score_list = IndScoreList::default();
        for score in iter {
            ind_score_list.push(score);
        }
        ind_score_list
    }
}

impl IndScoreList {
    fn push(&mut self, score: Box<dyn Score>) {
        self.delta += score.delta();
        self.score_list.push(score);
    }

    fn new(
        dist: &Dist,
        result_tokens: &HashSet<Token>,
        to_coincidence: &TokenToCoincidence,
    ) -> IndScoreList {
        let mut ind_score_list = IndScoreList::default();
        for (token, prob) in dist.token_and_prob.iter() {
            let score = SingleScore::new(result_tokens, token, to_coincidence, prob);
            ind_score_list.push(Box::new(score));
        }
        ind_score_list
    }
}

impl Person {
    pub fn name_points(
        &self,
        result_tokens: &HashSet<Token>,
        name_to_coincidence: &TokenToCoincidence,
    ) -> IndScoreList {
        // cmk0
        self.name_dist_list
            .iter()
            .map(|dist| DepScoreList::new(dist, result_tokens, name_to_coincidence))
            .map(|dep_score_list| Box::new(dep_score_list) as Box<dyn Score>)
            .collect()
    }

    pub fn city_points(
        &self,
        result_tokens: &HashSet<Token>,
        city_to_coincidence: &TokenToCoincidence,
    ) -> DepScoreList {
        // cmk0
        self.city_dist_list
            .iter()
            .map(|dist| DepScoreList::new(dist, result_tokens, city_to_coincidence))
            .map(|dep_score_list| Box::new(dep_score_list) as Box<dyn Score>)
            .collect()
    }
    // cmk be sure that init 0.0 is right
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
    show_work: String,
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
// cmk create good sample data
// cmk give users sliders for prob threshold? and priors? etc.
// cmk see the work doc for a mock up of the output
// cmk link: https://carlkcarlk.github.io/race-results/matcher/v0.1.0/index.html
// cmk I think this is OK. we use name_to_coincidence twice, but we could use it once.
// cmk will every ESR member be listed when looking at the NYC marathon because 'redmond', etc is rare in the results?
// cmk there must be a way to handle the city/vs not automatically.
// cmk create better sample data
// cmk hard to use from a phone (is there an easy way to access address list?)

// cmkdoc best on laptop
// cmkdoc many races let see all the results on one page, but some such as the NYC Marathon (with 47,000 runners) doesn't.
// cmkdoc the program assumes one result per line. Sometime when you cut and paste, a result will be split across many lines. I may
// cmkdoc able to fix this in the future for popular websites with a little more code. Please send me examples of race results are split across many lines.
// cmkdoc Mt./Mount/Mt Si but not NYC/New York City -- it splits on hyphens and spaces. Then slashes give alternatives for the single word.
// cmkdoc member list is tab or comma columns. Names and cities can have muliple words separated by spaces or -.
// cmkdoc finally alteratives are separated by / or &
// cmk test creating nice HTML when the name is Seattle Seattle living in Seattle
