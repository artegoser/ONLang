use crate::types::*;
use colored::*;
use json5;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::{self, Write};
use std::{thread, time};
pub struct Interpreter {
    input: String,
    vars: HashMap<String, Var>,
    pos: usize,
    scope: usize,
    scopes: Vec<Vec<String>>,
}

impl Interpreter {
    pub fn new(input: String) -> Self {
        Self {
            input,
            vars: HashMap::new(),
            pos: 1,
            scope: 0,
            scopes: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let obj: serde_json::Value = json5::from_str::<Value>(&self.input).unwrap_or_else(|_| {
            serde_yaml::from_str(&self.input)
                .expect("Your file format is invalid! (supported: json, json5 or yaml)")
        });
        let arr = obj.as_array().unwrap_or_else(|| {
            obj.get("main")
                .expect("Each program must contain a `{main: [..commands]}` object or be a command array ([..commands])")
                .as_array()
                .expect("The program must be an array of commands")
        });

        self.scopes.push(Vec::new());
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
                                self.error("Unsupported data type for the `print` argument, must be a string or an array");
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
                                self.error("Unsupported data type for the `println` argument, must be a string or an array");
                            }
                        },
                        "calc" => match value {
                            Value::Array(value) => {
                                if value.len() == 3 {
                                    return self.calc(value);
                                } else {
                                    self.error("Unsupported data type for the `calc` arguments, must be an array with three arguments");
                                }
                            }
                            _ => {
                                self.error("Unsupported data type for the `calc` arguments, must be an array with three arguments");
                            }
                        },
                        "comp" => match value {
                            Value::Array(value) => {
                                if value.len() == 3 {
                                    return self.comp(value);
                                } else {
                                    self.error("Unsupported data type for the `comp` arguments, must be an array with three arguments");
                                }
                            }
                            _ => {
                                self.error("Unsupported data type for the `comp` arguments, must be an array with three arguments");
                            }
                        },
                        "let" => match value {
                            Value::Object(value) => {
                                self.define(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `let` argument, must be an object");
                            }
                        },
                        "assign" => match value {
                            Value::Object(value) => {
                                self.assign(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `assign` argument, must be an object");
                            }
                        },

                        "var" => match value {
                            Value::String(value) => {
                                return self.get_var(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `var` argument, must be a string");
                            }
                        },
                        "isExist" => match value {
                            Value::String(value) => {
                                return Value::Bool(self.var_exists(value));
                            }
                            _ => {
                                self.error("Unsupported data type for the `isExist` argument, must be a string");
                            }
                        },
                        "delete" => match value {
                            Value::String(value) => {
                                self.delete(value, true);
                            }
                            _ => {
                                self.error("Unsupported data type for the `delete` argument, must be a string");
                            }
                        },
                        "input" => match value {
                            Value::String(value) => {
                                return self.input(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `input` argument, must be a string");
                            }
                        },
                        "sleep" => match value {
                            Value::Number(value) => {
                                self.sleep(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `sleep` argument, must be a number");
                            }
                        },
                        "if" => match value {
                            Value::Object(value) => {
                                return self.if_node(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `if` argument, must be an object");
                            }
                        },
                        "loop" => match value {
                            Value::Array(value) => {
                                return self.loop_cycle(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `loop cycle` argument, must be an array");
                            }
                        },
                        "scope" => match value {
                            Value::Array(value) => {
                                self.run_nodes(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `scope` argument, must be an array");
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
        io::stdout().flush().expect("Couldn't flush Stdout");
        io::stdin().read_line(&mut input).expect("Couldn't read from Stdin");
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
                            } else {
                                let name = self.run_nodes(else_nodes);
                                if name == "break" {
                                    return Value::String("break".to_string());
                                }
                                if name == "continue" {
                                    return Value::String("continue".to_string());
                                }
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
                            }
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
                        }
                    }
                },
                _ => {}
            },
            None => {
                self.error("if must have a body");
            }
        }
        Value::Null
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
        self.scope += 1;
        self.scopes.push(Vec::new());
        for command in arr {
            let to_do = self.eval_node(command);
            match to_do {
                Value::String(name) => return name,
                _ => {}
            }
        }
        self.delete_last_scope();
        self.scope -= 1;
        "end".to_string()
    }

    fn delete_last_scope(&mut self) {
        let vars = self.scopes[self.scope].clone();
        for name in vars {
            self.delete(&name, false);
        }
        self.scopes.remove(self.scope);
    }

    fn define(&mut self, vars: &Map<String, Value>) {
        for (name, value) in vars {
            if !self.var_exists(&name) {
                match value {
                    Value::Object(_) => {
                        let value = self.eval_node(value);
                        self.vars.insert(
                            name.clone(),
                            Var {
                                scope: self.scope,
                                body: value,
                            },
                        );
                        self.scopes[self.scope].push(name.clone())
                    }
                    _ => {
                        self.vars.insert(
                            name.clone(),
                            Var {
                                scope: self.scope,
                                body: value.clone(),
                            },
                        );
                        self.scopes[self.scope].push(name.clone())
                    }
                }
            } else {
                self.error(&format!("The variable {} already exist, use assign", name));
            }
        }
    }

    fn delete(&mut self, var_name: &String, panic: bool) {
        if self.var_exists(var_name) {
            self.vars.remove(var_name);
        } else {
            if panic {
                self.error(&format!(
                    "The variable {} does not exist and cannot be deleted",
                    var_name
                ));
            }
        }
    }

    fn get_var(&mut self, var_name: &String) -> Value {
        let var = self.vars.get(var_name);
        match var {
            Some(var) => return var.body.clone(),
            None => {
                self.error(&format!("The variable {} does not exist", var_name));
            }
        }
        Value::Null
    }

    fn get_var_scope(&mut self, var_name: &String) -> usize {
        let var = self.vars.get(var_name);
        match var {
            Some(var) => return var.scope,
            None => {
                self.error(&format!("The variable {} does not exist", var_name));
            }
        }
        0
    }

    fn assign(&mut self, vars: &Map<String, Value>) {
        for (name, value) in vars {
            let scope = self.get_var_scope(name);
            match value {
                Value::Object(_) => {
                    let value = self.eval_node(value);
                    self.vars
                        .insert(name.to_string(), Var { scope, body: value });
                }
                _ => {
                    self.vars.insert(
                        name.to_string(),
                        Var {
                            scope,
                            body: value.clone(),
                        },
                    );
                }
            }
        }
    }

    fn var_exists(&self, name: &String) -> bool {
        match self.vars.get(name) {
            Some(_) => true,
            None => false,
        }
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
                            "+" => return json!(op1 + op2),
                            "-" => return json!(op1 - op2),
                            "/" => return json!(op1 / op2),
                            "*" => return json!(op1 * op2),
                            "%" => return json!(op1 % op2),
                            "&" => return json!(op1 as i64 & op2 as i64),
                            "|" => return json!(op1 as i64 | op2 as i64),
                            "^" => return json!(op1 as i64 ^ op2 as i64),
                            "<<" => return json!((op1 as i64) << (op2 as i64)),
                            ">>" => return json!((op1 as i64) >> (op2 as i64)),
                            name => {
                                self.error(&format!(
                                    "Unsupported operation type for calculation: {}",
                                    name
                                ));
                            }
                        },
                        _ => {
                            self.error("Unexpected operator token");
                        }
                    }
                }
                Value::Object(_) => {
                    let op1 = Value::Number(op1.clone());
                    let op2 = self.eval_node(&op2.clone());
                    return self.calc(&vec![op1, operation.clone(), op2]);
                }
                _ => {
                    self.error("Unsupported operand type for calculation");
                }
            },
            Value::Object(_) => match op2 {
                Value::Number(_) => {
                    let op1 = self.eval_node(&op1.clone());
                    return self.calc(&vec![op1, operation.clone(), op2.clone()]);
                }
                Value::Object(_) => {
                    let op1 = self.eval_node(&op1.clone());
                    let op2 = self.eval_node(&op2.clone());
                    return self.calc(&vec![op1, operation.clone(), op2]);
                }
                _ => {
                    self.error("Unsupported operand type for calculation");
                }
            },
            _ => {
                self.error("Unsupported operand type for calculation");
            }
        }
        Value::Null
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
                    return self.comp(&vec![op1, operation.clone(), op2]);
                }
                _ => {
                    let op1 = self.eval_node(&op1.clone());
                    return self.comp(&vec![op1, operation.clone(), op2.clone()]);
                }
            },
            Value::Number(op1) => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    return self.comp(&vec![Value::Number(op1.clone()), operation.clone(), op2]);
                }
                Value::Number(op2) => {
                    let op1 = op1.as_f64().unwrap();
                    let op2 = op2.as_f64().unwrap();
                    match operation {
                        Value::String(operation) => match operation.as_str() {
                            "==" => return json!(op1 == op2),
                            "!=" => return json!(op1 != op2),
                            ">" => return json!(op1 > op2),
                            "<" => return json!(op1 < op2),
                            ">=" => return json!(op1 >= op2),
                            "<=" => return json!(op1 <= op2),
                            name => {
                                self.error(&format!(
                                    "Unsupported operation type for comparison: {}",
                                    name
                                ));
                            }
                        },
                        _ => {
                            self.error("Unexpected operator token");
                        }
                    }
                }
                _ => {
                    self.error("Unsupported operands types for comparison");
                }
            },
            Value::Bool(op1) => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    return self.comp(&vec![Value::Bool(op1.clone()), operation.clone(), op2]);
                }
                Value::Bool(op2) => match operation {
                    Value::String(operation) => match operation.as_str() {
                        "==" => return json!(op1 == op2),
                        "!=" => return json!(op1 != op2),
                        ">" => return json!(op1 > op2),
                        "<" => return json!(op1 < op2),
                        ">=" => return json!(op1 >= op2),
                        "<=" => return json!(op1 <= op2),
                        "&&" => return json!(*op1 && *op2),
                        "||" => return json!(*op1 || *op2),
                        name => {
                            self.error(&format!(
                                "Unsupported operation type for comparison: {}",
                                name
                            ));
                        }
                    },
                    _ => {
                        self.error("Unexpected operator token");
                    }
                },
                _ => {
                    self.error("Unsupported operands types for comparison");
                }
            },
            _ => match op2 {
                Value::Object(_) => {
                    let op2 = self.eval_node(&op2.clone());
                    return self.comp(&vec![op1.clone(), operation.clone(), op2]);
                }

                _ => match operation {
                    Value::String(operation) => match operation.as_str() {
                        "==" => return json!(op1 == op2),
                        "!=" => return json!(op1 != op2),
                        name => {
                            self.error(&format!(
                                "Unsupported operation type for comparison: {}",
                                name
                            ));
                        }
                    },
                    _ => {
                        self.error("Unexpected operator token");
                    }
                },
            },
        }
        Value::Null
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
