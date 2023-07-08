use itertools::Itertools;
use race_results::{delta_many_names, delta_one, log_odds, prob, NameToProb};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

fn split_name<'a>(name: &'a str, total_right: f32, re: &'a Regex) -> (Vec<&'a str>, Vec<f32>) {
    let name_list = re.split(name).collect::<Vec<&str>>();
    // Create a vector called right_list with the same length as name_list and with values = total_right/name_list.len()
    let right_list = name_list
        .iter()
        .map(|_| total_right / name_list.len() as f32)
        .collect_vec();

    (name_list, right_list)
}

fn main() -> io::Result<()> {
    // let sample_top = Path::new(r"C:\Users\carlk\OneDrive\Shares\RaceResults");
    // let members_file_name = sample_top.join("sample_members.tsv");
    // let results_with_city = sample_top.join("sample_results_withcity.txt");
    // let result_no_city = sample_top.join("sample_results_nocity.txt");
    let members_file_name = r"C:\Users\carlk\OneDrive\programs\MemberMatch\ESRMembers2012Dec.txt";
    let results_file_name = r"M:\projects\member_match\carnation2023results.txt";
    let re = Regex::new(r"[\-/ &]+").unwrap();
    let total_right = 0.6f32;
    let name_to_prob = NameToProb::default();

    let prob_member_in_race = 0.01;
    let result_count = read_lines(results_file_name)?.count();
    let prior_prob = prob_member_in_race / result_count as f32;
    let prior_points = log_odds(prior_prob);

    for member_list in read_lines(members_file_name)? {
        let line = member_list?;
        // cmk println!("line: '{:?}'", &line);
        let (first_name, last_name, city) = line.split('\t').collect_tuple().unwrap();
        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();
        // cmk println!("{} {} {}", first_name, last_name, city);

        let (first_name_list, first_right_list) = split_name(&first_name, total_right, &re);
        let (last_name_list, last_right_list) = split_name(&last_name, total_right, &re);

        // assert that every first_name_list, last_name, city contains only A-Z
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

        // cmk inefficient
        let city_count = read_lines(results_file_name)?
            .map(|result_line| {
                let result_line = result_line.unwrap().to_ascii_uppercase();
                result_line.contains(&city)
            })
            .filter(|x| *x)
            .count();
        let city_by_coincidence = (city_count + 1) as f32 / (result_count + 2) as f32;

        // cmk kind of crazy inefficient to score lines that have no tokens in common with this member
        for result_line in read_lines(results_file_name)? {
            let result_line = result_line?.to_ascii_uppercase();

            let contains_first_list: Vec<_> = first_name_list
                .iter()
                .map(|first_name| result_line.contains(first_name))
                .collect();
            let contains_last_list: Vec<_> = last_name_list
                .iter()
                .map(|last_name| result_line.contains(last_name))
                .collect();
            let contains_city = result_line.contains(&city);

            let first_name_points = delta_many_names(
                &contains_first_list,
                &first_name_list,
                &first_right_list,
                &name_to_prob,
            );

            let last_name_points = delta_many_names(
                &contains_last_list,
                &last_name_list,
                &last_right_list,
                &name_to_prob,
            );

            let city_name_points = delta_one(contains_city, city_by_coincidence, total_right);

            let post_points =
                prior_points + first_name_points + last_name_points + city_name_points;

            let post_prob = prob(post_points);

            if post_prob > 0.5 {
                println!(
                    "{:?} {:?} {} {:.2} {post_prob:.2} {result_line}",
                    first_name_list, last_name_list, city, post_points
                );
            }
        }
    }
    Ok(())
}

// cmk0 ING matches TUSING
