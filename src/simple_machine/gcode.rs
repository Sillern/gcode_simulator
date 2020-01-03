use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GCode {
    command: char,
    subcommand: usize,
    parameters: String,
}
type GCodeBlock = Vec<GCode>;
pub type GCodeProgram = HashMap<i32, GCodeBlock>;

pub fn parse(filepath: String) -> GCodeProgram {
    let contents =
        std::fs::read_to_string(filepath).expect("Something went wrong reading the file");

    // Detect empty lines and full line comments
    let invalid_line = Regex::new(r"(^\s*;)|(^$)|(^%)").unwrap();
    let clean_line = Regex::new(r"^([^;]*)").unwrap();
    let valid_commands =
        Regex::new(r"^(\s*(N)([0-9]+)\s)?\s?([A-Z])([-:.+0-9]+\b)(\s(.*)\b)?").unwrap();
    //let valid_parameters =
    //    Regex::new(r"([A-Z][-:.+0-9]+\s?|P'.*'\s?|\*[0-9]+\s?|!.*#\s?)").unwrap();

    let mut program = GCodeProgram::new();

    let mut linenumber = 0;
    for line in contents.lines().filter(|line| !invalid_line.is_match(line)) {
        let cleaned_line = match clean_line.captures(line) {
            Some(x) => match x.get(1) {
                Some(value) => value.as_str(),
                None => "",
            },
            None => "",
        };

        match valid_commands.captures(cleaned_line) {
            Some(x) => {
                linenumber = match x.get(3) {
                    Some(value) => match value.as_str().parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => linenumber,
                    },
                    None => linenumber,
                };

                let gcode = GCode {
                    command: match x.get(4) {
                        Some(value) => value.as_str().chars().next().unwrap(),
                        None => ' ',
                    },

                    subcommand: match x.get(5) {
                        Some(value) => match value.as_str().parse::<usize>() {
                            Ok(v) => v,
                            Err(_) => 0,
                        },
                        None => 0,
                    },

                    parameters: match x.get(7) {
                        Some(value) => value.as_str().to_string(),
                        None => "".to_string(),
                    },
                };

                program.insert(linenumber as i32, vec![gcode]);
                linenumber += 1;
            }
            None => (),
        }
    }

    return program;
}
