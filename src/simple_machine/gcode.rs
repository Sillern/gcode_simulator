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
                    minor: 0.0,
                    text_value: None
                }],
                result.1
            );
        }
    }

    #[test]
    fn test_with_one_param() {
        {
            let input = "N13 G0 X1.2";

            let result = parse_line(0, input).unwrap();

            assert_eq!(13, result.0);
            assert_eq!(
                vec![
                    GCode {
                        command: 'G',
                        major: 0,
                        minor: 0.0,
                        text_value: None
                    },
                    GCode {
                        command: 'X',
                        major: 1,
                        minor: 0.2,
                        text_value: None
                    }
                ],
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
                    minor: 0.2,
                    text_value: None
                }],
                result.1
            );
        }
        {
            let input = "O49(df)68 (dff (sdf) ) (OPTIONAL COMMENT) ; .";

            let result = parse_line(0, input).unwrap();

            assert_eq!(0, result.0);
            assert_eq!(
                vec![GCode {
                    command: 'O',
                    major: 4968,
                    minor: 0.0,
                    text_value: None
                }],
                result.1
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GCode {
    pub command: char,
    pub major: i32,
    pub minor: f32,
    text_value: Option<String>,
}
pub type GCodeBlock = Vec<GCode>;
pub type GCodeProgram = HashMap<i32, GCodeBlock>;

pub fn parse_line(linenumber: i32, line: &str) -> Option<(i32, GCodeBlock)> {
    // Detect comments
    let cleaned_line = match line.split(|c| c == ';' || c == '%').next() {
        Some(section) => {
            if section.len() != 0 {
                let mut comment_scope = 0;
                let mut cleaned_section: String = "".to_string();
                for c in section.chars() {
                    match c {
                        ')' => comment_scope -= 1,
                        '(' => comment_scope += 1,
                        _ => {
                            if comment_scope == 0 {
                                cleaned_section.push(c);
                            }
                        }
                    }
                }
                Some(cleaned_section)
            } else {
                None
            }
        }
        None => None,
    };

    let resolution: f32 = 10.0;

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
                                let mut parts = value.split('.');
                                let major = match parts.next() {
                                    Some(v) => match v.parse::<i32>() {
                                        Ok(v) => v,
                                        Err(_) => 0,
                                    },
                                    None => 0,
                                };
                                let minor = match parts.next() {
                                    Some(v) => match value.parse::<f32>() {
                                        Ok(v) => {
                                            ((v * resolution) - (major as f32 * resolution))
                                                / resolution
                                        }
                                        Err(_) => 0.0,
                                    },
                                    None => 0.0,
                                };

                                Some(GCode {
                                    command: command,
                                    major: major,
                                    minor: minor,
                                    text_value: None,
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
