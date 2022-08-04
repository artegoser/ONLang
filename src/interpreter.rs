use colored::*;
use json5;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::{self, Write};
use std::{thread, time};
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
        let obj = json5::from_str::<Value>(&self.input).expect("Your json is invalid!");
        let arr = obj.as_array().expect("Json must be an array!");

        for command in arr {
            self.eval_node(command);
            self.pos += 1;
        }
    }

    fn eval_node(&mut self, command: &Value) -> Value {
        match command {
            Value::Object(command) => {
                for (name, value) in command {
                    match name.as_str() {
                        "print" => match value {
                            Value::Array(value) => {
                                self.print(value, false);
                            }
                            Value::String(value) => {
                                self.print(&vec![Value::String(value.to_string())], false);
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
                                self.print(&vec![Value::String(value.to_string())], true);
                            }
                            _ => {
                                self.error("Unsupported data type for the println argument");
                            }
                        },
                        "calc" => match value {
                            Value::Array(value) => {
                                if value.len() == 3 {
                                    return self.calc(value);
                                } else {
                                    self.error("Unsupported data type for the calc arguments");
                                }
                            }
                            _ => {
                                self.error("Unsupported data type for the calc arguments");
                            }
                        },
                        "comp" => match value {
                            Value::Array(value) => {
                                if value.len() == 3 {
                                    return self.comp(value);
                                } else {
                                    self.error("Unsupported data type for the comp arguments");
                                }
                            }
                            _ => {
                                self.error("Unsupported data type for the comp arguments");
                            }
                        },
                        "let" => match value {
                            Value::Object(value) => {
                                return self.define(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the let argument");
                            }
                        },
                        "assign" => match value {
                            Value::Object(value) => {
                                return self.assign(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the let argument");
                            }
                        },
                        "var" => match value {
                            Value::String(value) => {
                                return self.get_var(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the let argument");
                            }
                        },
                        "input" => match value {
                            Value::String(value) => {
                                return self.input(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the input argument");
                            }
                        },
                        "sleep" => match value {
                            Value::Number(value) => {
                                self.sleep(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the sleep argument");
                            }
                        },
                        "if" => match value {
                            Value::Object(value) => {
                                return self.if_node(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the if argument");
                            }
                        },
                        "loop" => match value {
                            Value::Array(value) => {
                                return self.loop_cycle(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the loop cycle argument");
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
                "clear" => {
                    self.clear();
                }
                "break" => return Value::String("break".to_string()),
                "continue" => return Value::String("continue".to_string()),
                value => {
                    self.print(&vec![Value::String(value.to_string())], true);
                }
            },
            Value::Array(command) => {
                self.print(command, true);
            }
            _ => {
                self.error("Unsupported data type for the command");
            }
        }
        return Value::Null;
    }

    fn sleep(&self, value: &serde_json::Number) {
        let value = value.as_f64().unwrap() as u64;
        thread::sleep(time::Duration::from_millis(value));
    }

    fn clear(&self) {
        print!("{}[2J", 27 as char);
    }

    fn input(&self, value: &String) -> Value {
        let mut input = String::new();
        print!("{}", value);
        io::stdout().flush().unwrap_or_default();
        io::stdin().read_line(&mut input).unwrap_or_default();
        Value::String(input.trim_end().to_string())
    }

    fn if_node(&mut self, value: &Map<String, Value>) -> Value {
        let condition = self.eval_node(&value["condition"]);
        let nodes = &value.get("body");
        let else_nodes = &value.get("else");

        match nodes {
            Some(nodes) => match nodes {
                Value::Array(nodes) => match else_nodes {
                    Some(else_nodes) => match else_nodes {
                        Value::Array(else_nodes) => {
                            if condition == true {
                                let name = self.run_nodes(nodes);
                                if name == "break" {
                                    return Value::String("break".to_string());
                                }
                                if name == "continue" {
                                    return Value::String("continue".to_string());
                                }
                                return Value::Null;
                            } else {
                                let name = self.run_nodes(else_nodes);
                                if name == "break" {
                                    return Value::String("break".to_string());
                                }
                                if name == "continue" {
                                    return Value::String("continue".to_string());
                                }
                                return Value::Null;
                            };
                        }
                        _ => {
                            if condition == true {
                                let name = self.run_nodes(nodes);
                                if name == "break" {
                                    return Value::String("break".to_string());
                                }
                                if name == "continue" {
                                    return Value::String("continue".to_string());
                                }
                                return Value::Null;
                            }
                            return Value::Null;
                        }
                    },
                    None => {
                        if condition == true {
                            let name = self.run_nodes(nodes);
                            if name == "break" {
                                return Value::String("break".to_string());
                            }
                            if name == "continue" {
                                return Value::String("continue".to_string());
                            }
                            return Value::Null;
                        }
                        return Value::Null;
                    }
                },
                _ => return Value::Null,
            },
            None => {
                self.error("if must have a body");
                panic!()
            }
        }
    }

    fn loop_cycle(&mut self, value: &Vec<Value>) -> Value {
        loop {
            let name = self.run_nodes(value);
            if name == "break" {
                break Value::Null;
            }
            if name == "continue" {
                continue;
            }
        }
    }

    fn run_nodes(&mut self, arr: &Vec<Value>) -> String {
        for command in arr {
            let to_do = self.eval_node(command);
            match to_do {
                Value::String(name) => return name,
                _ => {}
            }
        }
        "end".to_string()
    }

    fn define(&mut self, vars: &Map<String, Value>) -> Value {
        for (name, value) in vars {
            match value {
                Value::Object(_) => {
                    let value = self.eval_node(value);
                    self.vars.insert(name.to_string(), value);
                }
                _ => {
                    self.vars.insert(name.to_string(), value.clone());
                }
            }
        }
        Value::Null
    }

    fn get_var(&mut self, var_name: &String) -> Value {
        let var = self.vars.get(var_name);
        match var {
            Some(var) => var.clone(),
            None => {
                self.error(&format!("The variable {} does not exist", var_name));
                panic!()
            }
        }
    }

    fn assign(&mut self, vars: &Map<String, Value>) -> Value {
        for (name, value) in vars {
            let var = self.vars.get(name);
            match var {
                Some(_) => match value {
                    Value::Object(_) => {
                        let value = self.eval_node(value);
                        self.vars.insert(name.to_string(), value);
                    }
                    _ => {
                        self.vars.insert(name.to_string(), value.clone());
                    }
                },
                None => {
                    self.error(&format!("The variable {} does not exist", name));
                    panic!();
                }
            }
        }
        Value::Null
    }

    fn calc(&mut self, value: &Vec<Value>) -> Value {
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
                        _ => {
                            self.error("Unexpected operator token");
                            panic!();
                        }
                    }
                }
                Value::Object(_) => {
                    let op1 = Value::Number(op1.clone());
                    let op2 = self.eval_node(&op2.clone());
                    self.calc(&vec![op1, operation.clone(), op2])
                }
                _ => {
                    self.error("Unsupported operand type for calculation");
                    panic!();
                }
            },
            Value::Object(_) => match op2 {
                Value::Number(_) => {
                    let op1 = self.eval_node(&op1.clone());
                    self.calc(&vec![op1, operation.clone(), op2.clone()])
                }
                Value::Object(_) => {
                    let op1 = self.eval_node(&op1.clone());
                    let op2 = self.eval_node(&op2.clone());
                    self.calc(&vec![op1, operation.clone(), op2])
                }
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

    fn comp(&mut self, value: &Vec<Value>) -> Value {
        let op1 = &value[0];
        let operation = &value[1];
        let op2 = &value[2];
        match op1 {
            Value::Object(_) => match op2 {
                Value::Object(_) => {
                    let op1 = self.eval_node(&op1.clone());
                    let op2 = self.eval_node(&op2.clone());
                    self.comp(&vec![op1, operation.clone(), op2])
                }
                _ => {
                    let op1 = self.eval_node(&op1.clone());
                    self.comp(&vec![op1, operation.clone(), op2.clone()])
                }
            },
            Value::Number(op1) => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    self.comp(&vec![Value::Number(op1.clone()), operation.clone(), op2])
                }
                Value::Number(op2) => {
                    let op1 = op1.as_f64().unwrap();
                    let op2 = op2.as_f64().unwrap();
                    match operation {
                        Value::String(operation) => match operation.as_str() {
                            "==" => json!(op1 == op2),
                            "!=" => json!(op1 != op2),
                            ">" => json!(op1 > op2),
                            "<" => json!(op1 < op2),
                            ">=" => json!(op1 >= op2),
                            "<=" => json!(op1 <= op2),
                            name => {
                                self.error(&format!(
                                    "Unsupported operation type for comparison: {}",
                                    name
                                ));
                                panic!();
                            }
                        },
                        _ => {
                            self.error("Unexpected operator token");
                            panic!();
                        }
                    }
                }
                _ => {
                    self.error("Unsupported operands types for comparison");
                    panic!();
                }
            },
            Value::Bool(op1) => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    self.comp(&vec![Value::Bool(op1.clone()), operation.clone(), op2])
                }
                Value::Bool(op2) => match operation {
                    Value::String(operation) => match operation.as_str() {
                        "==" => json!(op1 == op2),
                        "!=" => json!(op1 != op2),
                        ">" => json!(op1 > op2),
                        "<" => json!(op1 < op2),
                        ">=" => json!(op1 >= op2),
                        "<=" => json!(op1 <= op2),
                        "&&" => json!(*op1 && *op2),
                        "||" => json!(*op1 || *op2),
                        name => {
                            self.error(&format!(
                                "Unsupported operation type for comparison: {}",
                                name
                            ));
                            panic!();
                        }
                    },
                    _ => {
                        self.error("Unexpected operator token");
                        panic!();
                    }
                },
                _ => {
                    self.error("Unsupported operands types for comparison");
                    panic!();
                }
            },
            _ => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    self.comp(&vec![op1.clone(), operation.clone(), op2])
                }

                _ => match operation {
                    Value::String(operation) => match operation.as_str() {
                        "==" => json!(op1 == op2),
                        "!=" => json!(op1 != op2),
                        name => {
                            self.error(&format!(
                                "Unsupported operation type for comparison: {}",
                                name
                            ));
                            panic!();
                        }
                    },
                    _ => {
                        self.error("Unexpected operator token");
                        panic!();
                    }
                },
            },
        }
    }

    fn print(&mut self, args: &Vec<Value>, ln: bool) {
        for arg in args {
            self.print_one(arg);
        }
        if ln == true {
            println!();
        }
        io::stdout().flush().unwrap_or_default();
    }

    fn print_one(&mut self, arg: &Value) {
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
                let to_print = self.eval_node(&Value::Object(arg.clone()));
                self.print_one(&to_print);
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
