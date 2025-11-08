use serde_json::{Map, Value};

pub fn parse_vdf(input: &str) -> Value {
    let tokens = tokenize(input);
    let mut i = 0;
    let root = parse_object(&tokens, &mut i);
    Value::Object(root)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for c in input.chars() {
        match c {
            '"' => {
                if in_quote {
                    tokens.push(current.clone());
                    current.clear();
                }
                in_quote = !in_quote;
            }
            _ if in_quote => current.push(c),
            '{' | '}' if !in_quote => tokens.push(c.to_string()),
            _ => {}
        }
    }

    tokens
}

fn parse_object(tokens: &[String], i: &mut usize) -> Map<String, Value> {
    let mut map = Map::new();
    while *i < tokens.len() {
        match tokens[*i].as_str() {
            "}" => {
                *i += 1;
                break;
            }
            "{" => {
                *i += 1;
                continue;
            }
            key => {
                *i += 1;
                if *i < tokens.len() {
                    match tokens[*i].as_str() {
                        "{" => {
                            *i += 1;
                            let child = parse_object(tokens, i);
                            map.insert(key.to_string(), Value::Object(child));
                        }
                        "}" => {
                            *i += 1;
                        }
                        val => {
                            map.insert(key.to_string(), Value::String(val.to_string()));
                            *i += 1;
                        }
                    }
                }
            }
        }
    }
    map
}