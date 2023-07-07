use itertools::Itertools;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn read_lines<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(BufReader::new(File::open(path)?).lines())
}

fn split_name<'a>(name: &'a str, re: &'a Regex) -> (Vec<&'a str>, Vec<f32>) {
    let name_list = re.split(name).collect::<Vec<&str>>();
    let right = name_list.iter().map(|_| 1.0f32).collect::<Vec<f32>>();
    (name_list, right)
}

fn main() -> io::Result<()> {
    // let sample_top = Path::new(r"C:\Users\carlk\OneDrive\Shares\RaceResults");
    // let members_file_name = sample_top.join("sample_members.tsv");
    // let results_with_city = sample_top.join("sample_results_withcity.txt");
    // let result_no_city = sample_top.join("sample_results_nocity.txt");
    let members_file_name = r"C:\Users\carlk\OneDrive\programs\MemberMatch\ESRMembers2012Dec.txt";
    let results_file_name = r"M:\projects\member_match\carnation2023results.txt";
    let re = Regex::new(r"[\-/ &]").unwrap();

    for line in read_lines(members_file_name)? {
        let line = line?;
        println!("line: '{:?}'", &line);
        let (first_name, last_name, city) = line.split('\t').collect_tuple().unwrap();
        let first_name = first_name.to_uppercase();
        let last_name = last_name.to_uppercase();
        let city = city.to_uppercase();
        println!("{} {} {}", first_name, last_name, city);

        let (first_name_list, first_right) = split_name(&first_name, &re);
        let (last_name_list, last_right) = split_name(&last_name, &re);

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
    }
    Ok(())
}
