use colored::*;
use json5;
use serde_json::Value;
use std::collections::HashMap;
pub struct Interpreter {
    input: String,
    vars: HashMap<String, Value>,
    pos: usize,
}

impl Interpreter {
    pub fn new(input: String) -> Interpreter {
        Interpreter {
            input,
            vars: HashMap::new(),
            pos: 1,
        }
    }

    pub fn run(&mut self) {
        let mut obj = json5::from_str::<Value>(&self.input).unwrap();
        let arr = obj.as_array_mut().unwrap();

        for command in arr {
            match command {
                Value::Object(command) => {
                    for (name, value) in command {
                        match name.as_str() {
                            "print" => match value {
                                Value::Array(value) => {
                                    self.print(value, false);
                                }
                                Value::String(value) => {
                                    self.print(&mut vec![Value::String(value.to_string())], false);
                                }
                                _ => {
                                    self.error("Unsupported data type for the print argument");
                                }
                            },
                            "println" => match value {
                                Value::Array(value) => {
                                    self.print(value, true);
                                }
                                Value::String(value) => {
                                    self.print(&mut vec![Value::String(value.to_string())], true);
                                }
                                _ => {
                                    self.error("Unsupported data type for the println argument");
                                }
                            },
                            name => {
                                self.unk_token(&name);
                            }
                        }
                    }
                }
                Value::String(name) => match name.as_str() {
                    "Exit" => {
                        self.exit();
                    }
                    "ErrExit" => {
                        self.err_exit();
                    }
                    value => {
                        self.print(&mut vec![Value::String(value.to_string())], false);
                    }
                },
                Value::Array(command) => {
                    self.print(command, false);
                }
                _ => {
                    self.error("Unsupported data type for the command");
                }
            }

            self.pos += 1;
        }
    }
    fn print(&self, args: &mut Vec<Value>, ln: bool) {
        for arg in args {
            match arg {
                Value::Array(args) => {
                    print!("{}", serde_json::to_string_pretty(args).unwrap());
                }
                Value::String(arg) => {
                    print!("{}", arg);
                }
                Value::Bool(arg) => {
                    print!("{}", arg.to_string().blue());
                }
                Value::Number(arg) => {
                    print!("{}", arg.to_string().truecolor(180, 208, 143));
                }
                Value::Object(arg) => {
                    print!("{}", serde_json::to_string_pretty(arg).unwrap());
                }
                Value::Null => {
                    print!("{}", "null".blue());
                }
            }
            if ln == true {
                println!();
            }
        }
        if ln == false {
            println!();
        }
    }
    fn exit(&self) {
        println!("{}", "Programm finished with exit code: 0".green());
        std::process::exit(0);
    }
    fn err_exit(&self) {
        println!("{}", "Programm finished with exit code: 1".red());
        std::process::exit(1);
    }
    fn unk_token(&self, name: &str) {
        println!(
            "{} {} | {} {}",
            "Unexpected token name:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
        std::process::exit(1);
    }

    fn error(&self, name: &str) {
        println!(
            "{} {} | {} {}",
            "Error:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
        std::process::exit(1);
    }
}
