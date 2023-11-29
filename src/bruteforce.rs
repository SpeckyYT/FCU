use crate::util::*;
use std::{path::PathBuf, sync::{Mutex, Arc}, collections::HashSet};
use colored::Colorize;
use logos::Logos;
use rayon::prelude::*;
use lazy_static::lazy_static;


// handy function for bruteforcing
pub fn bruteforce(input: &str) {
    #[derive(Logos, Clone, Copy, Debug)]
    enum Token {
        #[token(r"\_")]
        All,   // [0-9a-z_]
        #[token(r"\x")]
        AlphaNumeric,   // [a-z0-9]
        #[token(r"\a")]
        Alphabetic,     // [a-z]
        #[token(r"\d")]
        Numeric,        // [0-9]
        #[token(r"\m")]
        Mails,          // 0 -> 100 + 1000
        #[regex(r"\([^)]+\)")]
        Specific,       // (abc)
        #[regex(".")]
        Text,
    }

    let mut lexer = Token::lexer(input);
    let mut lexed = vec![];
    
    while let Some(Ok(token)) = lexer.next() {
        lexed.push((token, lexer.slice()));    
    }
    lexed.reverse(); // cause of the "push / pop"

    fn recursive(mut remaining: Vec<(Token, &str)>, base_string: String) -> Vec<String> {
        let mut output = vec![];

        if let Some((token, string)) = remaining.pop() {
            macro_rules! new_output {
                ($input:expr) => {
                    for character in $input {
                        let new_base_string = base_string.clone() + &character.to_string();
                        output.append(&mut recursive(remaining.clone(), new_base_string.clone()));
                    }
                };
            }
            match token {
                Token::All => new_output!("abcdefghijklmnopqrstuvwxzy0123456789_".chars()),
                Token::AlphaNumeric => new_output!("abcdefghijklmnopqrstuvwxzy0123456789".chars()),
                Token::Alphabetic => new_output!("abcdefghijklmnopqrstuvwxzy".chars()),
                Token::Numeric => new_output!("0123456789".chars()),
                Token::Mails => new_output!((0..=100).chain(1000..=1000)),
                Token::Specific => new_output!(string[1..string.len()-1].chars()),
                Token::Text => new_output!(string.chars()),
            }
        }

        if output.is_empty() { output.push(base_string); }

        output
    }
    
    let all_combinations = recursive(lexed, String::new());

    lazy_static!{
        static ref ALL_IMAGES_STORED: Arc<Mutex<HashSet<String>>> = {
            let mut all_images = HashSet::new();
            crate::save::read_from_file(&mut all_images);
            Arc::new(Mutex::new(all_images))
        };
    }

    fn save_bruteforced() {
        crate::save::write(&ALL_IMAGES_STORED.lock().unwrap())
    }

    let images = all_combinations
        .into_par_iter()
        .map(|current_string|
            (
                download_image(
                    &relative_to_link(
                        &PathBuf::from(format!("{current_string}_{VERSION}.png"))
                    )
                ),
                current_string,
            )
        )
        .filter_map(|v| {
            if let (Some(bytes), name) = v {
                Some((bytes,name))
            } else {
                None
            }
        })
        .map(|v| {
            ALL_IMAGES_STORED.lock().unwrap().insert(v.1.clone());
            save_bruteforced();
            v
        })
        .collect::<Vec<_>>();

    println!("{}", LINES_SPLITTER.magenta());
    for (_, name) in images {
        println!("{}", name.green());
    }
    println!("{}", LINES_SPLITTER.magenta());
}
