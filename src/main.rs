use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

struct Bencode {}

impl Bencode {
    #[allow(dead_code)]
    fn decode(encoded_value: &str) -> serde_json::Value {
        // If encoded_value starts with a digit, it's a number
        let mut chars = encoded_value.chars();
        let first = chars.next();
        let last = chars.next_back();

        match (first, last) {
            (Some('i'), Some('e')) => {
                return Bencode::decode_bencoded_integer(encoded_value);
            }
            (Some('l'), Some('e')) => {
                return Bencode::decode_list(encoded_value).0;
            }
            (Some(c), _) => {
                if c.is_digit(10) {
                    return Bencode::decode_bencoded_string(encoded_value);
                } else {
                    panic!("Unhandled encoded value: {}", encoded_value)
                }
            }
            (None, _) => panic!("There is no argument"),
            // _ => panic!("Unhandled encoded value: {}", encoded_value),
        }
    }

    #[allow(dead_code)]
    fn decode_list(encoded_value: &str) -> (serde_json::Value, usize) {
        let mut vec: Vec<serde_json::Value> = Vec::new();
        let mut chars = encoded_value.chars();
        chars.next();
        let mut en_value = &encoded_value[1..encoded_value.len() - 1];

        while let Some(c) = chars.next() {
            // println!("c = {}, en_value = {}", c, en_value);
            if c.is_digit(10) {
                // println!("String");
                let colon_index = en_value.find(':').unwrap();
                let number_string = &en_value[..colon_index];
                let number = number_string.parse::<i64>().unwrap();
                let string = &en_value[colon_index + 1..colon_index + 1 + number as usize];
                vec.push(serde_json::Value::String(string.to_string()));

                let msg_len = colon_index + 1 + number as usize;
                en_value = &en_value[msg_len..];
                chars.nth(msg_len - 2);
            } else if c == 'i' {
                // println!("Integer");
                let end_index = en_value.find('e').unwrap();
                let number_string = &en_value[1..end_index];
                let number = number_string.parse::<i64>().unwrap();
                vec.push(serde_json::Value::Number(number.into()));
                // vec.push(serde_json::json!(number));

                let msg_len = end_index + 1 as usize;
                en_value = &en_value[msg_len..];
                chars.nth(msg_len - 2);
            } else if c == 'l' {
                // println!("List");
                let (value, length) = Bencode::decode_list(en_value);
                vec.push(value);

                let msg_len = length as usize;
                en_value = &en_value[msg_len..];
                chars.nth(msg_len - 2);
            } else if c == 'e' {
                let length = encoded_value.len() - chars.count() as usize;
                // println!("list ended : {}", &encoded_value[.. length]);
                return (serde_json::Value::Array(vec), length);
            } else {
                panic!("Unhandled encoded chacter: {}", c);
            }
        }
        // serde_json::Value::Array(vec)
        panic!("Unhandled encoded value: {}", encoded_value);
    }

    #[allow(dead_code)]
    fn decode_bencoded_string(encoded_value: &str) -> serde_json::Value {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return serde_json::Value::String(string.to_string());
    }

    #[allow(dead_code)]
    fn decode_bencoded_integer(encoded_value: &str) -> serde_json::Value {
        let number_string = &encoded_value[1..encoded_value.len() - 1];
        let number = number_string.parse::<i64>().unwrap();
        return serde_json::Value::Number(number.into());
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    // let (test, _) = decode_bencoded_list("l5:hello3:wow7:abcdef7i77ee");
    // let (test, _) = decode_bencoded_list("l5:hello3:wow7:abcdef7i77el5:helloi52eee");
    // println!("{}", test.to_string());

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = Bencode::decode(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
