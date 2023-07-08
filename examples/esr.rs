use itertools::Itertools;
use race_results::{delta_many_names, delta_one, delta_one_name, log_odds, prob, NameToProb};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::rc::Rc;

fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

fn split_name(name: &str, total_right: f32, re: &Regex) -> (Vec<String>, Vec<f32>) {
    let name_list = re.split(name).map(|s| s.to_owned()).collect::<Vec<_>>();
    // Create a vector called right_list with the same length as name_list and with values = total_right/name_list.len()
    let right_list = name_list
        .iter()
        .map(|_| total_right / name_list.len() as f32)
        .collect_vec();

    (name_list, right_list)
}

// cmk which is it a Person and a Member?
#[derive(Debug)]
struct Person {
    first_name_list: Vec<String>,
    first_right_list: Vec<f32>,
    last_name_list: Vec<String>,
    last_right_list: Vec<f32>,
    city: String,
    id: usize,
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

fn main() -> io::Result<()> {
    // let sample_top = Path::new(r"C:\Users\carlk\OneDrive\Shares\RaceResults");
    // let members_file_name = sample_top.join("sample_members.tsv");
    // let results_with_city = sample_top.join("sample_results_withcity.txt");
    // let result_no_city = sample_top.join("sample_results_nocity.txt");
    let members_file_name = r"C:\Users\carlk\OneDrive\programs\MemberMatch\ESRMembers2012Dec.txt";
    let results_file_name = r"M:\projects\member_match\carnation2023results.txt";
    let re = Regex::new(r"[\-/ &\t]+").unwrap();
    let total_right = 0.6f32;
    let name_to_prob = NameToProb::default();

    let prob_member_in_race = 0.01;

    // For ever token, in the results, find what fraction of the lines it occurs in (with smoothing).
    // (If a token is too common, we will not use it for initial matching.)
    let results_as_tokens: Vec<HashSet<String>> = read_lines(results_file_name)?
        .map(|result_line| {
            let result_line = result_line.unwrap().to_ascii_uppercase();
            let token_set: HashSet<String> = re
                .split(&result_line)
                .map(|s| s.to_owned())
                .filter(|token| !token.is_empty() && !token.chars().any(|c| c.is_ascii_digit()))
                .collect();
            println!("token_set={:?}", token_set);
            token_set
        })
        .collect();

    let result_count = results_as_tokens.len();
    let prior_prob = prob_member_in_race / result_count as f32;
    let prior_points = log_odds(prior_prob);

    let result_token_to_line_count =
        results_as_tokens
            .iter()
            .flatten()
            .fold(HashMap::new(), |mut acc, token| {
                *acc.entry(token.clone()).or_insert(0) += 1;
                acc
            });

    // print top 100 tokens
    let mut result_token_to_line_count_vec: Vec<_> = result_token_to_line_count.iter().collect();
    result_token_to_line_count_vec.sort_by_key(|(_token, count)| -**count);
    for (token, count) in result_token_to_line_count_vec.iter().take(100) {
        // for each token, in order of decreasing frequency, print its point value as a city and name, present and absent
        let city_conincidence = (*count + 1) as f32 / (result_count + 2) as f32;
        let city_points_contains = delta_one(true, city_conincidence, total_right);
        let city_points_absent = delta_one(false, city_conincidence, total_right);
        let name_points_contains = delta_one_name(true, token, total_right, &name_to_prob);
        let name_points_absent = delta_one_name(false, token, total_right, &name_to_prob);
        println!("{token}\t{count}\t{city_points_contains:.2}\t{city_points_absent:.2}\t{name_points_contains:.2}\t{name_points_absent:.2}");
    }

    let mut token_to_person_list: HashMap<String, Vec<Rc<Person>>> = HashMap::new();

    for (id, member_list) in (read_lines(members_file_name)?).enumerate() {
        let line = member_list?;
        let (first_name, last_name, city) = line.split('\t').collect_tuple().unwrap();
        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();

        let (first_name_list, first_right_list) = split_name(&first_name, total_right, &re);
        let (last_name_list, last_right_list) = split_name(&last_name, total_right, &re);

        // cmk assert that every first_name_list, last_name, city contains only A-Z
        for item in first_name_list.iter().chain(last_name_list.iter()) {
            for c in item.chars() {
                assert!(
                    c.is_ascii_alphabetic() || c == '.' || c == '\'',
                    "bad char in {:?}",
                    item
                );
            }
        }
        for c in city.chars() {
            if !(c.is_ascii_alphabetic() || c == ' ') {
                panic!("{}", format!("ascii={}, bad char in {:?}", c as u32, city));
            }
        }

        let person = Rc::new(Person {
            first_name_list,
            first_right_list,
            last_name_list,
            last_right_list,
            city,
            id,
        });
        // cmk is there a way to avoid cloning keys?
        for first_name in person.first_name_list.iter() {
            token_to_person_list
                .entry(first_name.clone())
                .or_insert(Vec::new())
                .push(person.clone());
        }
        for last_name in person.last_name_list.iter() {
            token_to_person_list
                .entry(last_name.clone())
                .or_insert(Vec::new())
                .push(person.clone());
        }
        token_to_person_list
            .entry(person.city.clone())
            .or_insert(Vec::new())
            .push(person.clone());
    }

    // // cmk inefficient
    // let city_count = read_lines(results_file_name)?
    //     .map(|result_line| {
    //         let result_line = result_line.unwrap().to_ascii_uppercase();
    //         result_line.contains(&city)
    //     })
    //     .filter(|x| *x)
    //     .count();
    // let city_by_coincidence = (city_count + 1) as f32 / (result_count + 2) as f32;

    // cmk kind of crazy inefficient to score lines that have no tokens in common with this member
    for result_line in read_lines(results_file_name)?.take(1) {
        let result_line = result_line?.to_ascii_uppercase();
        let result_list = re.split(&result_line).collect::<Vec<_>>();

        let person_set = result_list
            .iter()
            .filter_map(|token| token_to_person_list.get(*token))
            .flatten()
            .collect::<HashSet<_>>();

        if !person_set.is_empty() {
            println!("result_line={}", result_line);
            println!("{:?}", result_list);
            for person in person_set.iter() {
                println!("person={:?}", person);
            }
        }

        // let contains_first_list: Vec<_> = first_name_list
        //     .iter()
        //     .map(|first_name| result_line.contains(first_name))
        //     .collect();
        // let contains_last_list: Vec<_> = last_name_list
        //     .iter()
        //     .map(|last_name| result_line.contains(last_name))
        //     .collect();
        // let contains_city = result_line.contains(&city);

        // let first_name_points = delta_many_names(
        //     &contains_first_list,
        //     &first_name_list,
        //     &first_right_list,
        //     &name_to_prob,
        // );

        // let last_name_points = delta_many_names(
        //     &contains_last_list,
        //     &last_name_list,
        //     &last_right_list,
        //     &name_to_prob,
        // );

        // let city_name_points = delta_one(contains_city, city_by_coincidence, total_right);

        // let post_points = prior_points + first_name_points + last_name_points + city_name_points;

        // let post_prob = prob(post_points);

        // if post_prob > 0.5 {
        //     println!(
        //         "{:?} {:?} {} {:.2} {post_prob:.2} {result_line}",
        //         first_name_list, last_name_list, city, post_points
        //     );
        // }
    }
    Ok(())
}

// cmk0 ING matches TUSING
// cmk0 city: MILL CREEK should be treated like 1st name and last name but the number of names varies and the concidence is different.
// cmk0 only match tokens on rare cities, no the very, very common ones.
