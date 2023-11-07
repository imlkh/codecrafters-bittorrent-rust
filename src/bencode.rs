//! # Bencode
//!
//! decode bencoded valus to have intgers, strings, lists and, dictionaries
//!

use serde_json::{Map, Value};

pub trait Bencode {
    fn decode(&self) -> Value;
    fn decode_each(&self) -> (Value, &str);
    fn decode_integer(&self) -> (Value, &str);
    fn decode_string(&self) -> (Value, &str);
    fn decode_dictionary(&self) -> (Value, &str);
    fn decode_list(&self) -> (Value, &str);
}

impl Bencode for str {
    fn decode(&self) -> Value {
        let (value, encoded_remain) = self.decode_each();
        if !encoded_remain.is_empty() {
            println!("There is remaining encoded value : {}", encoded_remain);
        }
        value
    }

    fn decode_each(&self) -> (Value, &str) {
        let first = self.chars().next();

        match first {
            Some('i') => return self.decode_integer(),
            Some('l') => return self.decode_list(),
            Some('d') => return self.decode_dictionary(),
            Some(c) => {
                if c.is_digit(10) {
                    return self.decode_string();
                } else {
                    panic!("Unhandled encoded integer value: {}", self)
                }
            }
            None => panic!("There is no argument"),
        }
    }

    #[allow(dead_code)]
    fn decode_dictionary(&self) -> (Value, &str) {
        let mut map = Map::new();
        let mut en_value = &self[1..];

        while Some('e') != en_value.chars().next() {
            let (key, en_value_) = en_value.decode_each();
            let (value, en_value_) = en_value_.decode_each();
            en_value = en_value_;

            if let serde_json::Value::String(s) = key {
                map.insert(s, value);
            } else {
                panic!("key has to be a string");
            }
        }
        (serde_json::Value::Object(map), &en_value[1..])
    }

    fn decode_list(&self) -> (Value, &str) {
        let mut vec: Vec<Value> = Vec::new();
        let mut en_value = &self[1..];

        while let Some(c) = en_value.chars().next() {
            // println!("c = {}, en_value = {}", c, en_value);
            if c.is_digit(10) {
                // println!("String");
                let (value, encoded_next) = en_value.decode_string();
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'i' {
                // println!("Integer");
                let (value, encoded_next) = en_value.decode_integer();
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'l' {
                // println!("List");
                let (value, encoded_next) = en_value.decode_list();
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'd' {
                // println!("Dictionary");
                let (value, encoded_next) = en_value.decode_dictionary();
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'e' {
                return (Value::Array(vec), &en_value[1..]);
            } else {
                panic!("Unhandled encoded chacter: {}", c);
            }

            if en_value.is_empty() {
                panic!("There is no the ending 'e' symbol for outer list");
            }
        }
        panic!("Unhandled encoded value: {}", self);
    }

    fn decode_string(&self) -> (Value, &str) {
        let colon_index = self.find(':').unwrap();
        let number_string = &self[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &self[colon_index + 1..colon_index + 1 + number as usize];
        // consider storing it as Vec<u8>
        (
            Value::String(string.to_string()),
            &self[colon_index + 1 + number as usize..],
        )
    }

    fn decode_integer(&self) -> (Value, &str) {
        let end_index = self.find('e').unwrap();
        let number_string = &self[1..end_index];
        let number = number_string.parse::<i64>().unwrap();
        (
            Value::Number(number.into()),
            &self[end_index + 1..],
        )
    }
}

// pub struct Bencode {}

// impl Bencode {
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_integer() {
        assert_eq!(
            "i52e".decode().to_string(), 
            "52");
    }

    #[test]
    fn decode_list() {
        assert_eq!(
            "l5:hello3:wow7:abcdef7i77el5:helloi52eee".decode().to_string(), 
            "[\"hello\",\"wow\",\"abcdef7\",77,[\"hello\",52]]");
    }

    #[test]
    fn decode_dictionary() {
        assert_eq!(
            "d3:foo3:bar5:helloi52ee".decode().to_string(), 
            "{\"foo\":\"bar\",\"hello\":52}");
    }
}

