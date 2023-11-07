//! # Bencode
//!
//! decode bencoded valus to have intgers, strings, lists and, dictionaries
//!

use serde_json::{Map, Value};

pub struct Bencode {}

impl Bencode {
    pub fn decode(encoded_value: &str) -> Value {
        let (value, encoded_remain) = Bencode::decode_(encoded_value);
        if !encoded_remain.is_empty() {
            println!("There is remaining encoded value : {}", encoded_remain);
        }
        value
    }

    fn decode_(encoded_value: &str) -> (Value, &str) {
        let first = encoded_value.chars().next();

        match first {
            Some('i') => return Bencode::decode_integer(encoded_value),
            Some('l') => return Bencode::decode_list(encoded_value),
            Some('d') => return Bencode::decode_dictionary(encoded_value),
            Some(c) => {
                if c.is_digit(10) {
                    return Bencode::decode_string(encoded_value);
                } else {
                    panic!("Unhandled encoded integer value: {}", encoded_value)
                }
            }
            None => panic!("There is no argument"),
        }
    }

    #[allow(dead_code)]
    fn decode_dictionary(encoded_value: &str) -> (Value, &str) {
        let mut map = Map::new();
        let mut en_value = &encoded_value[1..];

        while Some('e') != en_value.chars().next() {
            let (key, en_value_) = Bencode::decode_(en_value);
            let (value, en_value_) = Bencode::decode_(en_value_);
            en_value = en_value_;

            if let serde_json::Value::String(s) = key {
                map.insert(s, value);
            } else {
                panic!("key has to be a string");
            }
        }
        (serde_json::Value::Object(map), &en_value[1..])
    }

    fn decode_list(encoded_value: &str) -> (Value, &str) {
        let mut vec: Vec<Value> = Vec::new();
        let mut en_value = &encoded_value[1..];

        while let Some(c) = en_value.chars().next() {
            // println!("c = {}, en_value = {}", c, en_value);
            if c.is_digit(10) {
                // println!("String");
                let (value, encoded_next) = Bencode::decode_string(en_value);
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'i' {
                // println!("Integer");
                let (value, encoded_next) = Bencode::decode_integer(en_value);
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'l' {
                // println!("List");
                let (value, encoded_next) = Bencode::decode_list(en_value);
                vec.push(value);
                en_value = encoded_next;
            } else if c == 'd' {
                // println!("Dictionary");
                let (value, encoded_next) = Bencode::decode_dictionary(en_value);
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
        panic!("Unhandled encoded value: {}", encoded_value);
    }

    fn decode_string(encoded_value: &str) -> (Value, &str) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        // consider storing it as Vec<u8>
        (
            Value::String(string.to_string()),
            &encoded_value[colon_index + 1 + number as usize..],
        )
    }

    fn decode_integer(encoded_value: &str) -> (Value, &str) {
        let end_index = encoded_value.find('e').unwrap();
        let number_string = &encoded_value[1..end_index];
        let number = number_string.parse::<i64>().unwrap();
        (
            Value::Number(number.into()),
            &encoded_value[end_index + 1..],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_list() {
        assert_eq!(
            Bencode::decode("l5:hello3:wow7:abcdef7i77el5:helloi52eee").to_string(), 
            "[\"hello\",\"wow\",\"abcdef7\",77,[\"hello\",52]]");
    }

    #[test]
    fn decode_dictionary() {
        assert_eq!(
            Bencode::decode("d3:foo3:bar5:helloi52ee").to_string(), 
            "{\"foo\":\"bar\",\"hello\":52}");
    }
}

