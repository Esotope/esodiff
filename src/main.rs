// Copyright (c) 2023 Jacob Allen Morris
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{env::args, path::PathBuf};

mod modules;

#[derive(Debug, Clone)]
pub struct Arguments {
    pub command: Option<String>,
    pub diff_file: Option<PathBuf>,
}

impl Arguments {
    fn new() -> Self {
        Arguments {
            command: None,
            diff_file: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ArgumentError {
    pub id: u32,
    pub index: u32,
}

#[derive(Debug, Clone)]
struct Argument {
    pub name: String,
    pub value_type: u32,
    pub needs_query: bool,
    pub value_as_string: Option<String>,
    pub value_as_u32: Option<u32>,
    pub value_as_bool: Option<bool>,
}

struct ArgumentInput {
    pub value_type: u32,
    pub value_as_string: Option<String>,
    pub value_as_u32: Option<u32>,
    pub value_as_bool: Option<bool>,
}

impl From<String> for ArgumentInput {
    fn from(value: String) -> Self {
        ArgumentInput {
            value_type: 1,
            value_as_string: Some(value.clone()),
            value_as_u32: None,
            value_as_bool: None,
        }
    }
}

impl From<u32> for ArgumentInput {
    fn from(value: u32) -> Self {
        ArgumentInput {
            value_type: 1,
            value_as_string: None,
            value_as_u32: Some(value.clone()),
            value_as_bool: None,
        }
    }
}

impl From<bool> for ArgumentInput {
    fn from(value: bool) -> Self {
        ArgumentInput {
            value_type: 1,
            value_as_string: None,
            value_as_u32: None,
            value_as_bool: Some(value.clone()),
        }
    }
}

impl Argument {
    fn new() -> Self {
        Argument {
            name: String::new(),
            value_type: 0,
            needs_query: false,
            value_as_string: None,
            value_as_u32: None,
            value_as_bool: None,
        }
    }

    fn set_argument_type(&mut self, item_type: &str) {
        match item_type {
            "diff" => {
                self.name = "diff".into();
                self.value_type = 1;
                self.needs_query = true;
            }
            "command" => {
                self.name = "command".into();
                self.value_type = 1;
            }
            _ => {}
        }
    }

    fn set_value<T: Into<ArgumentInput>>(&mut self, value: T) {
        let value: ArgumentInput = value.into();
        match value.value_type {
            1 => {
                self.value_type = 1;
                self.value_as_string = value.value_as_string;
            }
            2 => {
                self.value_type = 2;
                self.value_as_u32 = value.value_as_u32;
            }
            3 => {
                self.value_type = 3;
                self.value_as_bool = value.value_as_bool;
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.name = String::new();
        self.value_type = 0;
        self.needs_query = false;
        self.value_as_string = None;
        self.value_as_u32 = None;
        self.value_as_bool;
    }
}

fn handle_args() -> Result<Arguments, ArgumentError> {
    let args = args().collect::<Vec<String>>();
    let mut parsed_args: Vec<Argument> = Vec::new();
    let mut active_parsing_arg = Argument::new();
    let mut positional_arguments: u32 = 0;
    for (arg_id, arg) in args.iter().enumerate() {
        let arg = arg.to_owned();
        if arg_id > 0 {
            if !active_parsing_arg.needs_query {
                if arg.starts_with("-") {
                    if !arg.contains("=") {
                        match arg.as_str() {
                            "--diff" | "-d" => {
                                active_parsing_arg.set_argument_type("diff");
                            }
                            _ => {
                                return Err(ArgumentError {
                                    id: 1,
                                    index: arg_id as u32,
                                })
                            }
                        }
                    } else {
                        let root_arg = (&arg)
                            .split("=")
                            .map(|f| String::from(f))
                            .collect::<Vec<String>>()
                            .get(0)
                            .unwrap()
                            .to_owned();
                        let arg_input = (&arg)
                            .split("=")
                            .map(|f| String::from(f))
                            .collect::<Vec<String>>()
                            .get(1)
                            .unwrap()
                            .to_owned();
                        match root_arg.as_str() {
                            "--diff" | "-d" => {
                                active_parsing_arg.set_argument_type("diff");
                                active_parsing_arg.set_value(arg_input);
                                parsed_args.push(active_parsing_arg.clone());
                                active_parsing_arg.reset();
                            }
                            _ => {
                                return Err(ArgumentError {
                                    id: 1,
                                    index: arg_id as u32,
                                })
                            }
                        }
                    }
                } else {
                    if positional_arguments == 0 {
                        active_parsing_arg.set_argument_type("command");
                        active_parsing_arg.set_value(arg);
                        parsed_args.push(active_parsing_arg.clone());
                        active_parsing_arg.reset();
                        positional_arguments += 1;
                    } else {
                        return Err(ArgumentError {
                            id: 2,
                            index: arg_id as u32,
                        });
                    }
                }
            } else {
                let value_type = active_parsing_arg.value_type;
                match value_type {
                    1 => {
                        active_parsing_arg.set_value(arg);
                    }
                    _ => {}
                }
                parsed_args.push(active_parsing_arg.clone());
                active_parsing_arg.reset();
            }
        }
    }
    let mut final_args = Arguments::new();
    for item in parsed_args {
        match item.name.as_str() {
            "command" => {
                final_args.command = item.value_as_string;
            }
            "diff" => {
                final_args.diff_file = Some(PathBuf::from(item.value_as_string.unwrap()));
            }
            _ => {}
        }
    }
    Ok(final_args)
}

fn main() {
    let args = handle_args();
    if args.is_err() {
        panic!("Uhoh! Invalid arguments! Issue: {:?}", args.err().unwrap());
    } else {
        let args = args.unwrap();
        if (&args).command.as_ref().is_some() {
            match (&args).command.as_ref().unwrap().as_str() {
                "apply" => {
                    modules::apply(args);
                }
                _ => {
                    panic!("Uhoh! Invalid command!");
                }
            }
        } else {
            panic!("Uhoh! No command was specified!");
        }
    }
}
