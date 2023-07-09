use itertools::Itertools;
use race_results::{delta_many, delta_one, delta_one_name, log_odds, prob, TokenToCoincidence};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::rc::Rc;
use std::vec;

fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

// cmk move to Dist
fn split_name(name: &str, total_right: f32, re: &Regex) -> Dist {
    let name_list = re.split(name).map(|s| s.to_owned()).collect::<Vec<_>>();
    // Create a vector called right_list with the same length as name_list and with values = total_right/name_list.len()
    let right_list = name_list
        .iter()
        .map(|_| total_right / name_list.len() as f32)
        .collect_vec();

    Dist {
        token_and_prob: name_list.into_iter().zip(right_list).collect_vec(),
    }
}

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

    fn delta_names(&self, contains_list: &[bool], name_to_prob: &TokenToCoincidence) -> f32 {
        // cmk what if not found?
        // cmk why bother with collect?
        let prob_coincidence_list: Vec<_> = self
            .tokens()
            .iter()
            .map(|name| name_to_prob.prob(name.as_ref()))
            .collect();
        // cmk merge delta_many code to here
        delta_many(contains_list, &prob_coincidence_list, &self.probs())
    }

    // cmk0 make city_to_coincidence a struct
    fn delta_cities(
        &self,
        contains_list: &[bool],
        city_to_coincidence: &TokenToCoincidence,
    ) -> f32 {
        // cmk what if not found?
        // cmk why bother with collect?
        let prob_coincidence_list: Vec<_> = self
            .tokens()
            .iter()
            .map(|city| city_to_coincidence.prob(city.as_ref()))
            .collect();
        // cmk merge delta_many code to here
        delta_many(contains_list, &prob_coincidence_list, &self.probs())
    }
}

// cmk which is it a Person and a Member?
#[derive(Debug)]
struct Person {
    name_dist_list: Vec<Dist>,
    city_dist_list: Vec<Dist>,
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

struct LinePeople {
    line: String,
    max_prob: f32,
    person_prob_list: Vec<(Rc<Person>, f32)>,
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
    let name_to_conincidence = TokenToCoincidence::default_names();
    let stop_words_points = 3.0f32;
    let threshold_probability = 0.01f32;

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
            // println!("token_set={:?}", token_set);
            token_set
        })
        .collect();

    let result_count = results_as_tokens.len();
    let prior_prob = prob_member_in_race / result_count as f32;
    let prior_points = log_odds(prior_prob);
    let city_conincidence_default = 1f32 / (result_count + 2) as f32;

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
        default: city_conincidence_default,
    };
    for (token, count) in result_token_to_line_count_vec.iter() {
        // for each token, in order of decreasing frequency, print its point value as a city and name, present and absent
        let city_conincidence = (*count + 1) as f32 / (result_count + 2) as f32;
        city_to_coincidence
            .token_to_prob
            .insert(token.to_string(), city_conincidence);
        let city_points_contains = delta_one(true, city_conincidence, total_right);
        let name_points_contains = delta_one_name(true, token, total_right, &name_to_conincidence);
        // let city_points_absent = delta_one(false, city_conincidence, total_right);
        // let name_points_absent = delta_one_name(false, token, total_right, &name_to_prob);
        // println!("{token}\t{count}\t{city_points_contains:.2}\t{city_points_absent:.2}\t{name_points_contains:.2}\t{name_points_absent:.2}");
        if city_points_contains < stop_words_points {
            city_stop_words.insert(token.to_string());
        }
        if name_points_contains < stop_words_points {
            name_stop_words.insert(token.to_string());
        }
    }
    // println!("city_stop_words={:?}", city_stop_words);
    // println!("name_stop_words={:?}", name_stop_words);

    let mut token_to_person_list: HashMap<String, Vec<Rc<Person>>> = HashMap::new();

    for (id, member_list) in (read_lines(members_file_name)?).enumerate() {
        let line = member_list?;

        // cmk0 treat first and last more uniformly
        let (first_name, last_name, city) = line.split('\t').collect_tuple().unwrap();
        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();

        let first_dist = split_name(&first_name, total_right, &re);
        let last_dist = split_name(&last_name, total_right, &re);

        // cmk assert that every first_name_list, last_name, city contains only A-Z
        for item in first_dist.tokens().iter().chain(last_dist.tokens().iter()) {
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

        let name_dist_list = vec![first_dist, last_dist];
        let city_dist = Dist {
            token_and_prob: vec![(city.clone(), total_right)],
        };
        let city_dist_list: Vec<Dist> = vec![city_dist];
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

    // cmk kind of crazy inefficient to score lines that have no tokens in common with this member
    // cmk take 10
    let mut line_people_list: Vec<LinePeople> = Vec::new();
    for (result_line, result_tokens) in read_lines(results_file_name)?.zip(results_as_tokens)
    // .take(100)
    {
        let result_line = result_line?;

        let person_set = result_tokens
            .iter()
            .filter_map(|token| token_to_person_list.get(token))
            .flatten()
            .collect::<HashSet<_>>();

        // if !person_set.is_empty() {
        //     println!("result_line={}", result_line);
        //     println!("{:?}", result_tokens);
        //     for person in person_set.iter() {
        //         println!("person={:?}", person);
        //     }
        // }
        // optional LinePeople
        let mut line_people: Option<LinePeople> = None;
        for person in person_set.iter() {
            let person = *person;

            let name_points_sum: f32 = person
                .name_dist_list
                .iter()
                .map(|name_dist| {
                    // cmk combine these lines
                    let contains_list: Vec<_> = name_dist
                        .tokens()
                        .iter()
                        .map(|name| result_tokens.contains(name))
                        .collect();
                    name_dist.delta_names(&contains_list, &name_to_conincidence)
                })
                .sum();

            // cmk should name_to_prob and city_to_coincidence both be _prob or _coincidence?
            let city_points_sum: f32 = person
                .city_dist_list
                .iter()
                .map(|city_dist| {
                    // cmk combine these lines
                    let contains_list: Vec<_> = city_dist
                        .tokens()
                        .iter()
                        .map(|city| result_tokens.contains(city))
                        .collect();
                    city_dist.delta_cities(&contains_list, &city_to_coincidence)
                })
                .sum();

            let post_points = prior_points + name_points_sum + city_points_sum;

            let post_prob = prob(post_points);

            if post_prob > threshold_probability {
                // println!(
                //     "{:?} {:?} {} {:.2} {post_prob:.2} {result_line}",
                //     person.first_name_list, person.last_name_list, person.city, post_points
                // );
                if let Some(line_people) = &mut line_people {
                    line_people.max_prob = line_people.max_prob.max(post_prob);
                    line_people
                        .person_prob_list
                        .push((person.clone(), post_prob));
                } else {
                    line_people = Some(LinePeople {
                        line: result_line.clone(),
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

    // Sort by max_prob
    line_people_list.sort_by(|a, b| b.max_prob.partial_cmp(&a.max_prob).unwrap());
    for line_people in line_people_list.iter() {
        println!("{}", line_people.line);
        let mut person_prob_list = line_people.person_prob_list.clone();
        // sort by prob
        person_prob_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (person, prob) in person_prob_list.iter() {
            println!(
                "  {:?} {:?} {:.2}",
                // cmk if this is useful, make it a method
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
                prob
            );
        }
    }
    Ok(())
}

// cmk0 city: MILL CREEK should be treated like 1st name and last name but the number of names varies and the concidence is different.
