use crate::util::*;
use std::{path::PathBuf, sync::{Mutex, Arc}, collections::HashSet};
use colored::Colorize;
use logos::Logos;
// use rayon::prelude::*;
use futures::future::FutureExt; 
use futures::stream::{self, StreamExt};

const CONCURRENT_REQUESTS: usize = 128;

// handy function for bruteforcing
pub async fn bruteforce(input: &str, gay: bool) {
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

    let all_images_stored: Arc<Mutex<HashSet<String>>> = {
        let mut all_images = HashSet::new();
        crate::save::read_from_file(&mut all_images, gay);
        Arc::new(Mutex::new(all_images))
    };

    let all_images_found = Arc::new(Mutex::new(vec![]));

    let save_bruteforced = || {
        crate::save::write(&all_images_stored.lock().unwrap(), gay)
    };

    let size = all_combinations.len();

    stream::iter(all_combinations.into_iter().cycle().take(size))
    .map(|current_string| {
        let url = relative_to_link(
            &PathBuf::from(format!("{current_string}_{VERSION}.png")),
            gay,
        );
        download_image(url)
        .map(|bytes| (bytes, current_string))
    })
    .buffer_unordered(CONCURRENT_REQUESTS)
    .filter_map(|(bytes, current_string)| async {
        match bytes {
            Some(bytes) => Some((bytes, current_string)),
            None => None,
        }
    })
    .for_each(|(_bytes, current_string)| async {
        all_images_stored.lock().unwrap().insert(current_string.clone());
        all_images_found.lock().unwrap().push(current_string);
        save_bruteforced();
    })
    .await;

    println!("{}", LINES_SPLITTER.magenta());
    for name in all_images_found.lock().unwrap().iter() {
        println!("{}", name.green());
    }
    println!("{}", LINES_SPLITTER.magenta());
}
