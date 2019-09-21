use wasm_bindgen::prelude::*;
use pazzfraze::{Config, WordList};

const WORD_TEXT: &'static str = include_str!("../../words.txt");
static mut WORDS: Option<WordList> = None;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let word_list = WordList::new(WORD_TEXT.to_string()).expect("no words in wordlist");
    unsafe {
        WORDS=Some(word_list);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn gen(master: &str, domain: &str) -> String {
    let word_list = unsafe{
        WORDS.as_ref().expect("word list has not been initialized")
    };
    let conf = Config::new(word_list)
        .with_entropy(45.0)
        .with_style_pascal();
    let password = conf.gen(master.as_bytes(), domain.as_bytes());
    password
}
