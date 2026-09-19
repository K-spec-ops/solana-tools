use crate::*;

pub fn trimnw(s: &str)-> &str {
    let s: &str= s.trim_matches(|c: char| c.is_whitespace() || c== '\0');
    s
}

pub fn hello()-> () {
    println!("Hello world! I am utils!")
}