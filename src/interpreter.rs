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
            pos: 0,
        }
    }

    pub fn run(&mut self) {
        let mut obj = json5::from_str::<Value>(&self.input).unwrap();
        let arr = obj.as_array_mut().unwrap();

        for command in arr {
            for (name, value) in command.as_object().unwrap() {
                if name == "print" {
                    self.print(value);
                }
            }
        }
    }

    fn print(&self, args: &Value) {
        for arg in args.as_array().unwrap() {
            match arg {
                Value::Array(arg) => {
                    print!("{:#?}", arg);
                }
                Value::String(arg) => {
                    print!("{}", arg);
                }
                Value::Bool(arg) => {
                    print!("{}", arg);
                }
                Value::Number(arg) => {
                    print!("{}", arg);
                }
                Value::Object(arg) => {
                    print!("{:#?}", arg);
                }
                Value::Null => {
                    print!("null");
                }
            }
        }
        println!("");
    }
}
