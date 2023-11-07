//! # Bencode
//!
//! decode bencoded valus to have intgers, strings, lists and, dictionaries
//!

use serde_json::{Map, Value};
// use base64::{engine::general_purpose, Engine as _};

pub trait Bencode {
    fn decode(&self) -> Value;
    fn decode_each(&self) -> (Value, &Self);
    fn decode_integer(&self) -> (Value, &Self);
    fn decode_string(&self, is_hexadecimal: bool) -> (Value, &Self);
    fn decode_dictionary(&self) -> (Value, &Self);
    fn decode_list(&self) -> (Value, &Self);
}

impl Bencode for str {
    fn decode(&self) -> Value {
        let (value, encoded_remain) = self.decode_each();
        if !encoded_remain.is_empty() {
            eprintln!("There is remaining encoded value : {}", encoded_remain);
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
                    return self.decode_string(false);
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
                let (value, encoded_next) = en_value.decode_string(false);
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
        panic!("Unhandled encoded value: {}", en_value);
    }

    fn decode_string(&self, _is_hexadecimal: bool) -> (Value, &str){
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
        // u32::try_from(1).ok();
        // self.iter().take_while()
        // self.iter().map_while()

        // let number_string: String = iter
        //     .by_ref()
        //     .take_while(|&&b| b as char == 'e')
        //     .map(|&b| b as char)
        //     .collect();

impl Bencode for [u8] {
    fn decode(&self) -> Value {
        let (value, encoded_remain) = self.decode_each();
        if !encoded_remain.is_empty() {
            eprintln!("There is remaining encoded value, its length : {}", encoded_remain.len());
        }
        value
    }
    fn decode_each(&self) -> (Value, &[u8]){
        let first = self.iter().next();

        match first {
            Some(b'i') => return self.decode_integer(),
            Some(b'l') => return self.decode_list(),
            Some(b'd') => return self.decode_dictionary(),
            Some(&b) => {
                let c = b as char;
                if c.is_digit(10) {
                    return self.decode_string(false);
                } else {
                    panic!("Unhandled encoded integer value, its length : {}", self.len())
                }
            }
            None => panic!("There is no argument"),
        }
    }
    fn decode_integer(&self) -> (Value, &[u8]){
        let mut iter = self.iter();
        if let Some(&b) = iter.next() {
            if b != b'i' {
                panic!("It's not for integer");
            }
        }
        let number_counter = iter
            .take_while(|&&b| b != b'e')
            .count();
        let number_string = String::from_utf8((&self[1..number_counter + 1]).into()).unwrap();
        let number = number_string.parse::<i64>().unwrap();
        (
            Value::Number(number.into()),
            &self[number_counter + 2..],
        )
    }
    fn decode_string(&self, is_hexadecimal: bool) -> (Value, &[u8]){
        let colon_index = self.iter()
            // .inspect(|&&b| eprintln!("{}", b as char))
            .take_while(|&&b| b != b':')
            .count();
        let number_string = String::from_utf8((&self[..colon_index]).into()).unwrap();
        let number = number_string.parse::<i64>().unwrap();
        if is_hexadecimal {
            eprintln!("byte lengt is {}", number);
            let pieces = &self[colon_index + 1..colon_index + 1 + number as usize];
            // let string = String::from_utf8((&pieces[..20]).into()).unwrap(); 
            let string: String = pieces.iter().map(|b| format!("{:02x}",b)).collect();
            (
                Value::String(string.to_string()),
                &self[colon_index + 1 + number as usize..],
            )

        } else {
            let string = String::from_utf8((&self[colon_index + 1..colon_index + 1 + number as usize]).into()).unwrap();
            (
                Value::String(string.to_string()),
                &self[colon_index + 1 + number as usize..],
            )
        }

    }
    fn decode_dictionary(&self) -> (Value, &[u8]){
        let mut map = Map::new();
        let mut en_value = &self[1..];

        while Some(&b'e') != en_value.iter().next() {
            let (key, en_value_) = en_value.decode_each();

            if let Value::String(s) = key {
                if s == "pieces" {
                    let (value, en_value_) = en_value_.decode_string(true);
                    en_value = en_value_;
                    map.insert(s, value);
                } else {
                    let (value, en_value_) = en_value_.decode_each();
                    en_value = en_value_;
                    map.insert(s, value);
                }
            } else {
                panic!("key has to be a string");
            }
        }
        (serde_json::Value::Object(map), &en_value[1..])
    }

    fn decode_list(&self) -> (Value, &[u8]){
        let mut vec: Vec<Value> = Vec::new();
        let mut en_value = &self[1..];

        while let Some(&b) = en_value.iter().next() {
            if (b as char).is_digit(10) {
                // println!("String");
                let (value, encoded_next) = en_value.decode_string(false);
                vec.push(value);
                en_value = encoded_next;
            } else if b == b'i' {
                // println!("Integer");
                let (value, encoded_next) = en_value.decode_integer();
                vec.push(value);
                en_value = encoded_next;
            } else if b == b'l' {
                // println!("List");
                let (value, encoded_next) = en_value.decode_list();
                vec.push(value);
                en_value = encoded_next;
            } else if b == b'd' {
                // println!("Dictionary");
                let (value, encoded_next) = en_value.decode_dictionary();
                vec.push(value);
                en_value = encoded_next;
            } else if b == b'e' {
                return (Value::Array(vec), &en_value[1..]);
            } else {
                panic!("Unhandled encoded chacter: {}", b as char);
            }

            if en_value.is_empty() {
                panic!("There is no the ending 'e' symbol for outer list");
            }
        }
        panic!("Unhandled encoded value, its length : {}", en_value.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_integer_byte() {
        assert_eq!(
            b"i-5222e".decode().to_string(), 
            "-5222");
    }

    #[test]
    fn decode_string_byte() {
        assert_eq!(
            b"5:hello".decode().to_string(), 
            "\"hello\"");
    }

    #[test]
    fn decode_list_byte() {
        assert_eq!(
            b"l5:hello3:wow7:abcdef7i77el5:helloi52eee".decode().to_string(), 
            "[\"hello\",\"wow\",\"abcdef7\",77,[\"hello\",52]]");
    }

    #[test]
    fn decode_dictionary_byte() {
        assert_eq!(
            b"d3:foo3:bar5:helloi52ee".decode().to_string(), 
            "{\"foo\":\"bar\",\"hello\":52}");
    }

    /////////////////////////////////////////

    #[test]
    fn decode_string() {
        assert_eq!(
            "i52e".decode().to_string(), 
            "52");
    }

    #[test]
    fn decode_integer() {
        assert_eq!(
            "5:hello".decode().to_string(), 
            "\"hello\"");
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

