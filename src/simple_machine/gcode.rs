use itertools::Itertools;
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
                    major: 0,
                    minor: 0,
                    raw_value: None
                }],
                result.1
            );
        }
    }

    #[test]
    fn test_comments() {
        {
            let input = "%N11 X0";

            let result = parse_line(0, input);

            assert_eq!(None, result);
        }
        {
            let input = ";N11 X0";

            let result = parse_line(0, input);

            assert_eq!(None, result);
        }
        {
            let input = "; N11 X0";

            let result = parse_line(0, input);

            assert_eq!(None, result);
        }
        {
            let input = "N13 X2.2 ;N11 X0";

            let result = parse_line(0, input).unwrap();

            assert_eq!(13, result.0);
            assert_eq!(
                vec![GCode {
                    command: 'X',
                    major: 2,
                    minor: 2,
                    raw_value: None
                }],
                result.1
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GCode {
    pub command: char,
    pub major: usize,
    pub minor: usize,
    raw_value: Option<String>,
}
type GCodeBlock = Vec<GCode>;
pub type GCodeProgram = HashMap<i32, GCodeBlock>;

pub fn parse_line(linenumber: i32, line: &str) -> Option<(i32, GCodeBlock)> {
    // Detect comments
    let cleaned_line = match line.split(|c| c == ';' || c == '%').next() {
        Some(section) => {
            if section.len() == 0 {
                None
            } else {
                Some(section)
            }
        }
        None => None,
    };

    let mut gcodeblock = GCodeBlock::new();

    match cleaned_line {
        Some(valid_line) => {
            let mut first_token = true;
            let mut validated_linenumber = linenumber;

            for token in valid_line.split_whitespace() {
                // Both must be valid
                let parameter = match token.get(0..1) {
                    Some(name) => {
                        let command = match name.chars().next() {
                            Some(command) => command,
                            None => ' ',
                        };
                        match token.get(1..) {
                            Some(value) => {
                                let parts = value
                                    .split('.')
                                    .into_iter()
                                    .map(|v| match v.parse::<usize>() {
                                        Ok(v) => v,
                                        Err(_) => 0,
                                    })
                                    .collect::<Vec<usize>>();

                                Some(GCode {
                                    command: command,
                                    major: match parts.get(0) {
                                        Some(v) => *v,
                                        None => 0,
                                    },
                                    minor: match parts.get(1) {
                                        Some(v) => *v,
                                        None => 0,
                                    },
                                    raw_value: None,
                                })
                            }
                            None => None,
                        }
                    }
                    None => None,
                };

                match parameter {
                    Some(parameter) => {
                        if first_token && parameter.command == 'N' {
                            first_token = false;
                            // An N-command was stated, override the default linenumber
                            validated_linenumber = parameter.major as i32;
                        } else {
                            gcodeblock.push(parameter);
                        }
                    }
                    None => (),
                }
            }
            Some((validated_linenumber, gcodeblock))
        }
        None => None,
    }
}

pub fn parse(filepath: String) -> GCodeProgram {
    let contents =
        std::fs::read_to_string(filepath).expect("Something went wrong reading the file");

    let mut program = GCodeProgram::new();

    let mut linenumber = 0;
    for line in contents.lines() {
        match parse_line(linenumber as i32, line) {
            Some((line, gcode)) => {
                program.insert(line, gcode);
                linenumber += 1;
            }
            None => (),
        }
    }

    return program;
}
