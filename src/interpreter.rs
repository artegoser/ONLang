use colored::*;
use json5;
use serde_json::{json, Value};
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
            self.eval_node(command);
        }

        self.pos += 1;
    }

    fn eval_node(&self, command: &mut Value) -> Value {
        match command {
            Value::Object(command) => {
                for (name, value) in command {
                    match name.as_str() {
                        "print" => {
                            match value {
                                Value::Array(value) => {
                                    self.print(value, false);
                                }
                                Value::String(value) => {
                                    self.print(&mut vec![Value::String(value.to_string())], false);
                                }
                                _ => {
                                    self.error("Unsupported data type for the print argument");
                                }
                            }
                            return Value::Null;
                        }
                        "println" => {
                            match value {
                                Value::Array(value) => {
                                    self.print(value, true);
                                }
                                Value::String(value) => {
                                    self.print(&mut vec![Value::String(value.to_string())], true);
                                }
                                _ => {
                                    self.error("Unsupported data type for the println argument");
                                }
                            }
                            return Value::Null;
                        }
                        "calc" => {
                            match value {
                                Value::Array(value) => {
                                    if value.len() == 3 {
                                        return self.calc(value);
                                    } else {
                                        return Value::Null;
                                    }
                                }
                                _ => {
                                    self.error("Unsupported data type for the println argument");
                                }
                            }
                            return Value::Null;
                        }
                        name => {
                            self.unk_token(&name);
                            return Value::Null;
                        }
                    }
                }
                Value::Null
            }
            Value::String(name) => {
                match name.as_str() {
                    "Exit" => {
                        self.exit();
                    }
                    "ErrExit" => {
                        self.err_exit();
                    }
                    value => {
                        self.print(&mut vec![Value::String(value.to_string())], false);
                    }
                }
                Value::Null
            }
            Value::Array(command) => {
                self.print(command, false);
                Value::Null
            }
            _ => {
                self.error("Unsupported data type for the command");
                Value::Null
            }
        }
    }

    fn calc(&self, value: &mut Vec<Value>) -> Value {
        let op1 = &value[0];
        let operation = &value[1];
        let op2 = &value[2];
        match op1 {
            Value::Number(op1) => match op2 {
                Value::Number(op2) => {
                    let op1 = op1.as_f64().unwrap();
                    let op2 = op2.as_f64().unwrap();
                    match operation {
                        Value::String(operation) => match operation.as_str() {
                            "+" => json!(op1 + op2),
                            "-" => json!(op1 - op2),
                            "/" => json!(op1 / op2),
                            "*" => json!(op1 * op2),
                            "%" => json!(op1 % op2),
                            "&" => json!(op1 as i64 & op2 as i64),
                            "|" => json!(op1 as i64 | op2 as i64),
                            "^" => json!(op1 as i64 ^ op2 as i64),
                            "<<" => json!((op1 as i64) << (op2 as i64)),
                            ">>" => json!((op1 as i64) >> (op2 as i64)),
                            name => {
                                self.error(&format!(
                                    "Unsupported operation type for calculation: {}",
                                    name
                                ));
                                panic!();
                            }
                        },
                        name => {
                            self.error(&format!(
                                "Unsupported operation type for calculation: {}",
                                name
                            ));
                            panic!();
                        }
                    }
                }
                Value::Object(_) => self.calc(&mut vec![
                    serde_json::Value::Number(op1.clone()),
                    operation.clone(),
                    self.eval_node(&mut op2.clone()),
                ]),
                _ => {
                    self.error("Unsupported operand type for calculation");
                    panic!();
                }
            },
            Value::Object(_) => match op2 {
                Value::Number(_) => self.calc(&mut vec![
                    self.eval_node(&mut op1.clone()),
                    operation.clone(),
                    op2.clone(),
                ]),
                Value::Object(_) => self.calc(&mut vec![
                    self.eval_node(&mut op1.clone()),
                    Value::String(operation.to_string()),
                    self.eval_node(&mut op2.clone()),
                ]),
                _ => {
                    self.error("Unsupported operand type for calculation");
                    panic!();
                }
            },
            _ => {
                self.error("Unsupported operand type for calculation");
                panic!();
            }
        }
    }

    fn print(&self, args: &mut Vec<Value>, ln: bool) {
        for arg in args {
            self.print_one(arg);
            if ln == true {
                println!();
            }
        }
        if ln == false {
            println!();
        }
    }

    fn print_one(&self, arg: &Value) {
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
                self.print_one(&self.eval_node(&mut Value::Object(arg.clone())));
            }
            Value::Null => {
                print!("{}", "null".blue());
            }
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
            "\n{} {} | {} {}",
            "Unexpected token name:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
        std::process::exit(1);
    }

    fn error(&self, name: &str) {
        println!(
            "\n{} {} | {} {}",
            "Error:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
        std::process::exit(1);
    }
}
