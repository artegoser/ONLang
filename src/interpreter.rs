use crate::types::*;
use colored::*;
use json5;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::{fs, thread, time};
pub struct Interpreter {
    commands: Vec<Value>,
    vars: HashMap<String, Var>,
    pos: usize,
    scope: usize,
    scopes: Vec<Vec<String>>,
}

impl Interpreter {
    pub fn new(file_path: String) -> Self {
        match Path::new(&file_path)
            .extension()
            .and_then(OsStr::to_str)
            .expect("The file must have the extension (.yaml, .json, .json5, .onla or .conla)")
        {
            "yaml" => {
                let file_input = fs::read_to_string(&file_path).expect("File reading error");
                let obj: serde_json::Value =
                    serde_yaml::from_str(&file_input).unwrap_or_else(|x| {
                        match x.location() {
                            Some(location) => {
                                eprintln!(
                                    "{file_path}:{}:{} --> {x}",
                                    location.column(),
                                    location.line()
                                );
                            }
                            None => {
                                eprintln!("{}", x);
                            }
                        }

                        std::process::exit(1);
                    });
                let commands = obj.as_array().unwrap_or_else(|| {
                    obj.get("main")
                        .expect("Each program must contain a `{main: [..commands]}` object or be a command array ([..commands])")
                        .as_array()
                        .expect("The program must be an array of commands")
                });
                Self {
                    commands: commands.clone(),
                    vars: HashMap::new(),
                    pos: 1,
                    scope: 0,
                    scopes: Vec::new(),
                }
            }
            "conla" => {
                let file_input = File::open(file_path).expect("File reading error");
                let obj: serde_json::Value = rmp_serde::from_read(file_input)
                    .expect(".conla file (MessagePack) is invalid! ");
                let commands = obj.as_array().unwrap_or_else(|| {
                    obj.get("main")
                        .expect("Each program must contain a `{main: [..commands]}` object or be a command array ([..commands])")
                        .as_array()
                        .expect("The program must be an array of commands")
                });
                Self {
                    commands: commands.clone(),
                    vars: HashMap::new(),
                    pos: 1,
                    scope: 0,
                    scopes: Vec::new(),
                }
            }
            _ => {
                let file_input = fs::read_to_string(&file_path).expect("File reading error");
                let obj: serde_json::Value =
                    json5::from_str::<Value>(&file_input).unwrap_or_else(|x| {
                        eprintln!("{file_path}{x}");
                        std::process::exit(1);
                    });
                let commands = obj.as_array().unwrap_or_else(|| {
                    obj.get("main")
                        .expect("Each program must contain a `{main: [..commands]}` object or be a command array ([..commands])")
                        .as_array()
                        .expect("The program must be an array of commands")
                });

                Self {
                    commands: commands.clone(),
                    vars: HashMap::new(),
                    pos: 1,
                    scope: 0,
                    scopes: Vec::new(),
                }
            }
        }
    }

    pub fn compress(&mut self, output_path: String) {
        let mut output = File::create(output_path).expect("Failed to create output file");
        output
            .write_all(
                &rmp_serde::encode::to_vec(&self.commands)
                    .expect("Error when compressing onlang to .conla"),
            )
            .expect("Error when writing to file");
        println!("Compressed");
    }

    pub fn convert(&self, format: String, output_path: String) {
        match format.as_str() {
            "yaml" => {
                self.convert_to_yaml(output_path);
            }
            "json" => {
                self.convert_to_json(output_path);
            }
            "json5" => {
                self.convert_to_json5(output_path);
            }
            _ => {
                self.error("The conversion format is not supported");
            }
        }
    }

    fn convert_to_yaml(&self, output_path: String) {
        let mut output = File::create(output_path).expect("Failed to create output file");
        write!(
            output,
            "{}",
            serde_yaml::to_string(&self.commands).expect("Error when convert to yaml")
        )
        .expect("Error when writing to file");

        println!("Converted");
    }

    fn convert_to_json(&self, output_path: String) {
        let mut output = File::create(output_path).expect("Failed to create output file");
        write!(
            output,
            "{}",
            serde_json::to_string(&self.commands).expect("Error when convert to json")
        )
        .expect("Error when writing to file");

        println!("Converted");
    }

    fn convert_to_json5(&self, output_path: String) {
        let mut output = File::create(output_path).expect("Failed to create output file");
        write!(
            output,
            "{}",
            json5::to_string(&self.commands).expect("Error when convert to json5")
        )
        .expect("Error when writing to file");

        println!("Converted");
    }

    pub fn run(&mut self) {
        self.scopes.push(Vec::new());
        let length = self.commands.len();
        for i in 0..length {
            let command = &self.commands[i].clone();
            self.eval_node(command);
            self.pos = i;
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
                                return self.get_var(value).body.clone();
                            }
                            _ => {
                                self.error("Unsupported data type for the `var` argument, must be a string");
                            }
                        },
                        "ref" => match value {
                            Value::String(value) => {
                                return self.variable_reference(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `ref` argument, must be a string");
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
                        "fn" => match value {
                            Value::Object(value) => {
                                self.define_fn(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `fn` argument, must be an object");
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
                        "arr" => match value {
                            Value::Array(value) => {
                                return self.calc_arr(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `arr` argument, must be an array");
                            }
                        },
                        "toString" => {
                            return serde_json::Value::String(
                                serde_json::to_string_pretty(&self.eval_node(value))
                                    .expect("Some error"),
                            )
                        }

                        "obj" => match value {
                            Value::Object(value) => {
                                return self.calc_obj(value);
                            }
                            _ => {
                                self.error("Unsupported data type for the `obj` argument, must be an array");
                            }
                        },

                        "return" => return json!({ "return": value }),
                        name => match value {
                            Value::Array(value) => {
                                return self.run_fn(name.to_string(), value);
                            }
                            _ => {
                                self.unk_token(&name);
                            }
                        },
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
            val => {
                println!("{}", serde_json::to_string_pretty(val).unwrap());
                self.error("Unsupported data type for the command");
            }
        }
        return Value::Null;
    }

    fn calc_arr(&mut self, value: &Vec<Value>) -> Value {
        Value::Array(
            value
                .into_iter()
                .map(|val| match val {
                    Value::Object(_) => self.eval_node(val),
                    _ => val.clone(),
                })
                .collect(),
        )
    }

    fn calc_obj(&mut self, value: &Map<String, Value>) -> Value {
        let result: Value = value
            .into_iter()
            .map(|(k, v)| match v {
                Value::Object(_) => (k.clone(), self.eval_node(v)),
                _ => (k.clone(), v.clone()),
            })
            .collect();
        result
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
        io::stdin()
            .read_line(&mut input)
            .expect("Couldn't read from Stdin");
        Value::String(input.trim_end().to_string())
    }

    fn run_fn(&mut self, name: String, args: &Vec<Value>) -> Value {
        let function = self.get_var(&name);
        match function.var_type {
            VarTypes::Variable => {
                self.error(&format!("`{}` not a function", name));
            }
            VarTypes::Function => {
                let function = function.body.clone();
                let real_len = args.len();
                let func_args = function.get("args").unwrap().as_array().unwrap();
                if real_len == func_args.len() {
                    for i in 0..real_len {
                        let argname = &func_args[i];
                        match argname {
                            Value::String(argname) => {
                                self.create_var(&format!("{}.{}", name, argname), &args[i], false);
                            }
                            _ => self.error("Argument name must be a string"),
                        }
                    }

                    let name = self.run_nodes(function.get("body").unwrap().as_array().unwrap());
                    match name {
                        Value::Object(name) => {
                            return name.get("return").unwrap().clone();
                        }
                        _ => return Value::Null,
                    }
                } else {
                    self.error(&format!(
                        "`{}` must have {} arguments, but {} is specified",
                        name,
                        func_args.len(),
                        real_len
                    ));
                }
            }
        }
        Value::Null
    }
    fn define_fn(&mut self, value: &Map<String, Value>) {
        let name = &value.get("name");
        let body = &value.get("body");
        let args = &value.get("args");

        match name {
            Some(name) => match name {
                Value::String(name) => match body {
                    Some(body) => match body {
                        Value::Array(body) => match args {
                            Some(args) => match args {
                                Value::Array(args) => {
                                    // let args: Vec<String> = args.iter().map(|val| format!("{}.{}", name, val)).collect();
                                    self.vars.insert(
                                        name.clone(),
                                        Var {
                                            scope: self.scope,
                                            body: json!({"args":args, "body":body}),
                                            var_type: VarTypes::Function,
                                        },
                                    );
                                }
                                _ => self.error("Arguments must be an array of strings"),
                            },
                            None => self
                                .error("Each function must have an array of arguments or an empty array instead of them"),
                        },
                        _ => self.error("Body must be an array of commands"),
                    },
                    None => self.error("Each function must have a body"),
                },
                _ => self.error("Name must be a string"),
            },
            None => self.error("Function must have a name"),
        }
    }

    fn if_node(&mut self, value: &Map<String, Value>) -> Value {
        let condition = self.eval_node(
            &value
                .get("condition")
                .expect("`if` must have a `condition` argument"),
        );
        let nodes = &value.get("body");
        let else_nodes = &value.get("else");

        match nodes {
            Some(nodes) => match nodes {
                Value::Array(nodes) => match else_nodes {
                    Some(else_nodes) => match else_nodes {
                        Value::Array(else_nodes) => {
                            if condition == true {
                                let name = self.run_nodes(nodes);
                                return name;
                            } else {
                                let name = self.run_nodes(else_nodes);
                                return name;
                            };
                        }
                        _ => {
                            if condition == true {
                                let name = self.run_nodes(nodes);
                                return name;
                            }
                        }
                    },
                    None => {
                        if condition == true {
                            let name = self.run_nodes(nodes);
                            return name;
                        }
                    }
                },
                _ => {}
            },
            None => {
                self.error("`if` must have a body");
            }
        }
        Value::Null
    }

    fn loop_cycle(&mut self, value: &Vec<Value>) -> Value {
        loop {
            let name = self.run_nodes(value);
            match name {
                Value::String(name) => {
                    if name == "break" {
                        break Value::Null;
                    } else if name == "continue" {
                        continue;
                    }
                }
                Value::Object(_) => {
                    return name;
                }
                _ => {}
            }
        }
    }

    fn run_nodes(&mut self, arr: &Vec<Value>) -> Value {
        self.enter_the_scope();
        for command in arr {
            let to_do = self.eval_node(command);
            match to_do {
                Value::String(name) => {
                    if name == "break" || name == "continue" {
                        self.exit_from_scope();
                        return Value::String(name);
                    }
                }
                Value::Object(name) => {
                    let check = name.get("return");
                    match check {
                        Some(check) => match check {
                            Value::Object(_) => {
                                let result = json!({"return": self.eval_node(check)});
                                self.exit_from_scope();
                                return result;
                            }
                            _ => {
                                self.exit_from_scope();
                                return json!({"return": check.clone()});
                            }
                        },
                        None => {}
                    }
                }
                _ => {}
            }
        }
        self.exit_from_scope();
        Value::String("end".to_string())
    }

    fn enter_the_scope(&mut self) {
        self.scope += 1;
        self.scopes.push(Vec::new());
    }

    fn exit_from_scope(&mut self) {
        self.delete_last_scope();
        self.scope -= 1;
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
            self.create_var(name, value, true);
        }
    }
    fn create_var(&mut self, name: &String, value: &Value, panic: bool) {
        if !self.var_exists(&name) {
            match value {
                Value::Object(_) => {
                    let value = self.eval_node(value);
                    self.vars.insert(
                        name.clone(),
                        Var {
                            scope: self.scope,
                            body: value,
                            var_type: VarTypes::Variable,
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
                            var_type: VarTypes::Variable,
                        },
                    );
                    self.scopes[self.scope].push(name.clone())
                }
            }
        } else {
            if panic {
                self.error(&format!("The variable {} already exist, use assign", name));
            } else {
                self.assign_var(name, value);
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

    fn get_var(&mut self, var_name: &String) -> &Var {
        let var = self.vars.get(var_name);
        match var {
            Some(var) => return var,
            None => {
                self.error(&format!("The variable {} does not exist", var_name));
                panic!();
            }
        }
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
            self.assign_var(name, value);
        }
    }
    fn assign_var(&mut self, name: &String, value: &Value) {
        let scope = self.get_var_scope(name);
        match value {
            Value::Object(_) => {
                let value = self.eval_node(value);
                self.vars.insert(
                    name.to_string(),
                    Var {
                        scope,
                        body: value,
                        var_type: VarTypes::Variable,
                    },
                );
            }
            _ => {
                self.vars.insert(
                    name.to_string(),
                    Var {
                        scope,
                        body: value.clone(),
                        var_type: VarTypes::Variable,
                    },
                );
            }
        }
    }
    fn var_exists(&self, name: &String) -> bool {
        match self.vars.get(name) {
            Some(_) => true,
            None => false,
        }
    }

    fn variable_reference(&mut self, name: &String) -> Value {
        if self.var_exists(name) {
            return json!({ "var": name });
        } else {
            self.error(&format!(
                "The variable {} does not exist, the reference is invalid",
                name
            ));
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
        panic!(
            "\n{} {} | {} {}",
            "Unexpected token name:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
    }

    fn error(&self, name: &str) {
        panic!(
            "\n{} {} | {} {}",
            "Error:".red(),
            name.bold().black(),
            "pos:".green(),
            self.pos
        );
    }
}
