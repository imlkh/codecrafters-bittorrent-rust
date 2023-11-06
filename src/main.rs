use serde_json::{Map, Value};
use std::env;

use std::io;
use std::io::Read;
use std::fs::File;

use std::str;

// Available if you need it!
// use serde_bencode

pub struct Bencode {}

impl Bencode {
    #[allow(dead_code)]
    fn decode(encoded_value: &str) -> Value {
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

        while Some('e') != en_value.chars().next()  {
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

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    // let test = Bencode::decode("l5:hello3:wow7:abcdef7i77ee");
    // let test = Bencode::decode("l5:hello3:wow7:abcdef7i77el5:helloi52eee");
    // let test = Bencode::decode("d3:foo3:bar5:helloi52ee");
    // println!("{}", test.to_string());

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = Bencode::decode(encoded_value);
        // println!("{}", decoded_value.to_string());

        Ok(())
    } else if command == "info" {
        let file_path = &args[2];
    //
        // convert Vec<u8> to &str
        let mut f = File::open(file_path)?;
        // let mut buffer : Vec<u8> = Vec::new();
        // let mut buffer = [0u8; 200];
        let mut buffer : Vec<u8> = Vec::new();
        // let mut buffer =  String::new();
        f.read_to_end(&mut buffer)?;
        // let n = f.read(&mut buffer)?;
        // println!("n = {n}");
        // println!("buffer.len() = {}",buffer.len());
    
        // one byte for two hexa-decimal values

        // match f.read_to_string(&mut buffer) {
        // match f.read_to_string(&mut buffer) {
            // Ok(l) => println!("ok : {}", l),
            // Err(l) => println!("error : {}", l)
        // }

        // println!("buffer[0] = {}", buffer[0] as char);
        // println!("buffer[1] = {}", buffer[1] as char);
        // let i = 212; println!("buffer[{}] = {}, {} as char", i, buffer[i], buffer[i] as char);
        // let i = 181; println!("buffer[{}] = {}, {} as char", i, buffer[i], buffer[i] as char);
        // let i = 182; println!("buffer[{}] = {}", i, buffer[i]);

        let counter = buffer
            .iter()
            .enumerate()
            .filter(|(i,&b)| !b.is_ascii() || b.is_ascii_control())
            .count();
        println!("counter = {}", counter);

        let non_ascii: Vec<usize> = buffer
            .iter()
            .enumerate()
            .filter(|(_,&b)| !b.is_ascii() || b.is_ascii_control())
            .map(|(i,_)| i)
            // .inspect(|i| print!("{}, ", i))
            .collect();

        // println!("");
        // buffer.iter().for_each(|&b| println!("{}", b as char));
        // let str : String = buffer[..210]
        let str : String = buffer
        // buffer[..220]
        // buffer[..213]
        // buffer
            .iter()
            .enumerate()
            .filter(|(i,&b)| b.is_ascii() && !b.is_ascii_control())
            // .for_each(|&b| print!("{}", b as char));
            // .for_each(|&b| print!("{}", b.to ));
            // .inspect(|(i,_)| print!("{},", i))
            .map(|(i,&b)| b as char)
            .collect();
        // println!("{}", str);


        // println!("{}", &str[..172]);
        // println!("{}", &str[..98]);

        let mut str = (&str[..98]).to_owned();
        str += "e";

        // println!("str : {}", str);
       
        // char::from_u32(i)
        // println!("buffer : {}", buffer);
        let encoded_value = &str;
        // let char = str::from_utf8(&buffer[..4]).unwrap();
        // let encoded_value = str::from_utf8(&buffer).unwrap();
        // buffer.iter().map(|b| str::from_utf8(b).unwrap()).collect();
        // let encoded_value = str::from_utf8(&buffer).unwrap();
        // println!("char : {}", char);
        // println!("encoded_value : {}", encoded_value);
        // println!("buffer.len() = {}, buffer[0] = {}", buffer.len(), &buffer[0]);
        let decoded_value = Bencode::decode(encoded_value);
        // buffer.iter().for_each(|b| println!("{}",b));
        // buffer.iter().for_each(|b|
        //     {
        //         if b.is_ascii() {
        //             println!("{}",b.to_ascii_uppercase())
        //         } else {
        //             println!("{} : not ascii",b)
        //         }
        //
        //     });
        println!("{}", decoded_value.to_string());

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
