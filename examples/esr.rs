use std::io;

use race_results::{read_lines, Config, IncludeCity};

fn main() -> io::Result<()> {
    // let sample_top = Path::new(r"C:\Users\carlk\OneDrive\Shares\RaceResults");
    // let members_file_name = sample_top.join("sample_members.tsv");
    // let results_with_city = sample_top.join("sample_results_withcity.txt");
    // let result_no_city = sample_top.join("sample_results_nocity.txt");
    let members_file_name = r"C:\Users\carlk\OneDrive\programs\MemberMatch\ESRMembers2012Dec.txt";
    let results_file_name = r"M:\projects\member_match\carnation2023results.txt";
    let include_city = IncludeCity::Yes;
    let member_lines = read_lines(members_file_name)?.map(|line| line.unwrap());
    let result_lines = read_lines(results_file_name)?.map(|line| line.unwrap());
    // cmk this doesn't look good
    let line_list = Config::default().find_matches(member_lines, result_lines, include_city);

    let line_list = line_list.unwrap();
    for line in line_list.iter() {
        println!("{}", line);
    }
    Ok(())
}
