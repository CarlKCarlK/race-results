use itertools::Itertools;
use race_results::{delta_many, delta_one, delta_one_name, log_odds, prob, TokenToCoincidence};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use std::iter::repeat;
use std::path::Path;
use std::rc::Rc;
use std::vec;

fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

const NICKNAMES_STR: &str = include_str!("nicknames.txt");

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
    ) -> Self {
        assert!(
            total_nickname <= total_right / 2.0,
            "Expect total nickname to be <= than half total_right"
        );
        assert!(
            (0.0..=1.0).contains(&total_right),
            "Expect total_right to be between 0 and 1"
        );
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
        for item in dist.tokens().iter() {
            for c in item.chars() {
                assert!(
                    c.is_ascii_alphabetic() || c == '.' || c == '\'',
                    "bad char in {:?}",
                    item
                );
            }
        }
        dist
    }

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
    // cmk there should be a tokenize struct, etc.
    let prob_member_in_race = 0.01;
    let total_right = 0.6f32;
    let total_nickname = 0.1f32;
    let name_to_coincidence = TokenToCoincidence::default_names();
    let stop_words_points = 3.0f32;
    let threshold_probability = 0.01f32;

    let re = Regex::new(r"[\-/ &\t]+").unwrap();

    let mut name_to_nickname_set = HashMap::<String, HashSet<String>>::new();
    // cmk add something to this???
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
    // println!("name_to_nickname_set={:?}", name_to_nickname_set);

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
        token_to_prob: BTreeMap::new(),
        default: city_coincidence_default,
    };
    for (token, count) in result_token_to_line_count_vec.iter() {
        // for each token, in order of decreasing frequency, print its point value as a city and name, present and absent
        let city_coincidence = (*count + 1) as f32 / (result_count + 2) as f32;
        city_to_coincidence
            .token_to_prob
            .insert(token.to_string(), city_coincidence);
        let city_points_contains = delta_one(true, city_coincidence, total_right);
        let name_points_contains = delta_one_name(true, token, total_right, &name_to_coincidence);
        // let city_points_absent = delta_one(false, city_coincidence, total_right);
        // let name_points_absent = delta_one_name(false, token, total_right, &name_to_coincidence);
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

        // cmk treat first and last more uniformly
        // cmk show a nice error if the line is not tab-separated, three columns
        // cmk println!("line={:?}", line);
        let (first_name, last_name, city) = line.split('\t').collect_tuple().unwrap();
        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();

        let first_dist = Dist::split_token(
            &first_name,
            total_right,
            &re,
            &name_to_nickname_set,
            total_nickname,
        );
        let last_dist = Dist::split_token(
            &last_name,
            total_right,
            &re,
            &name_to_nickname_set,
            total_nickname,
        );
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
            .collect();

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

        let mut line_people: Option<LinePeople> = None;
        for person in person_set.iter() {
            let person = *person;

            let name_points_sum = person.name_points(&result_tokens, &name_to_coincidence);
            let city_points_sum = person.city_points(&result_tokens, &city_to_coincidence);

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
        }
    }
    Ok(())
}

// cmk should O'Neil tokenize to ONEIL?
// cmk be sure there is a way to run without matching on city
// cmk what about people with two-part first names?
