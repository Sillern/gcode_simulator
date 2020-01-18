use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_params() {
        {
            let input = "N11 X0";

            let result = parse_line(0, input).unwrap();

            assert_eq!(11, result.0);
            assert_eq!(
                vec![GCode {
                    command: 'X',
                    subcommand: (0, 0),
                    parameters: "".to_string()
                }],
                result.1
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GCode {
    pub command: char,
    pub subcommand: (usize, usize),
    parameters: String,
}
type GCodeBlock = Vec<GCode>;
pub type GCodeProgram = HashMap<i32, GCodeBlock>;

pub fn parse_line(linenumber: i32, line: &str) -> Option<(i32, GCodeBlock)> {
    let valid_commands =
        Regex::new(r"^(\s*(N)([0-9]+)\s)?\s?([A-Z])([-+.0-9]+\b)(\s(.*))?").unwrap();
    match valid_commands.captures(line) {
        Some(x) => {
            // An N-command was stated, override the default linenumber
            let validated_linenumber = match x.get(3) {
                Some(value) => match value.as_str().parse::<i32>() {
                    Ok(v) => v,
                    Err(_) => linenumber,
                },
                None => linenumber,
            };

            let gcode = GCode {
                command: match x.get(4) {
                    Some(value) => match value.as_str().chars().next() {
                        Some(c) => c,
                        None => ' ',
                    },
                    None => ' ',
                },

                subcommand: match x.get(5) {
                    Some(value) => {
                        let parts = value
                            .as_str()
                            .split(".")
                            .into_iter()
                            .map(|v| match v.parse::<usize>() {
                                Ok(v) => v,
                                Err(_) => 0,
                            })
                            .collect::<Vec<usize>>();
                        (
                            match parts.get(0) {
                                Some(v) => *v,
                                None => 0,
                            },
                            match parts.get(1) {
                                Some(v) => *v,
                                None => 0,
                            },
                        )
                    }
                    None => (0, 0),
                },

                parameters: match x.get(7) {
                    Some(value) => value.as_str().to_string(),
                    None => "".to_string(),
                },
            };

            Some((validated_linenumber, vec![gcode]))
        }
        None => None,
    }
}

pub fn parse(filepath: String) -> GCodeProgram {
    let contents =
        std::fs::read_to_string(filepath).expect("Something went wrong reading the file");

    // Detect empty lines and full line comments
    let invalid_line = Regex::new(r"(^\s*;)|(^$)|(^%)").unwrap();
    let clean_line = Regex::new(r"^([^;]*)").unwrap();

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

        match parse_line(linenumber as i32, cleaned_line) {
            Some((line, gcode)) => {
                program.insert(line, gcode);
                linenumber += 1;
            }
            None => (),
        }
    }

    return program;
}
