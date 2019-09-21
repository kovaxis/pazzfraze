//! Generate nice-looking passwords out of a master password and a domain name.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha512};

const DEFAULT_ENTROPY: f64 = 48.0;
const SALT: &'static [u8] = b"\x8e\xbf\x78\x79\xc9\xe9\xac\xe7\x91\xb6\xb4\xc9\x2b\x9b\x50\xe7\x60\xe5\x76\x01\x73\x59\x49\x9c\x74\x18\x4e\x01\x38\xcc\x7c\x69\x03\x46\x9d\xc1\xbd\xf0\x28\x99\xab\xa9\xda\xf2\x82\x0b\xbe\x3d\xc4\x8c\xea\x56\x03\x35\x6c\xa3\x05\x86\x59\x9e\xec\xe8\xbe\xa4";

/// A list of words to choose from when generating passwords.
#[derive(Debug, Clone)]
pub struct WordList {
    text: String,
    words: Vec<(usize, usize)>,
}
impl WordList {
    pub fn new(text: String) -> Option<WordList> {
        let words = text
            .split_whitespace()
            .map(|str| {
                let idx = (str.as_ptr() as usize) - (text.as_ptr() as usize);
                (idx, str.len())
            })
            .collect::<Vec<_>>();
        if words.len() > 0 {
            Some(WordList { text, words })
        } else {
            None
        }
    }
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
    pub fn word(&self, idx: usize) -> &str {
        let (start, len) = self.words[idx];
        &self.text[start..start + len]
    }
}

/// How to join the different words to form a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Style {
    /// `MyGreatPassword`
    Pascal,
    /// `myGreatPassword`
    Camel,
    /// For example, if the string is `_`:
    /// `my_great_password`
    Concat(String),
}
impl Default for Style {
    fn default() -> Self {
        Style::Pascal
    }
}
impl Style {
    fn push(&self, word: &str, into: &mut String, first: bool, _last: bool) {
        match self {
            Style::Pascal => {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    for ch in first.to_uppercase() {
                        into.push(ch);
                    }
                }
                into.push_str(chars.as_str());
            }
            Style::Camel => {
                let mut chars = word.chars();
                if !first {
                    if let Some(first) = chars.next() {
                        for ch in first.to_uppercase() {
                            into.push(ch);
                        }
                    }
                }
                into.push_str(chars.as_str());
            }
            Style::Concat(sep) => {
                if !first {
                    into.push_str(sep);
                }
                into.push_str(word);
            }
        }
    }
}

fn get_hash(hasher: &mut Sha512) -> [u8; 64] {
    let mut hash = [0; 64];
    hash.copy_from_slice(&hasher.result_reset());
    hash
}

fn secure_hash(master: &[u8], domain: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    //Hash the master password a few thousand times
    hasher.input(SALT);
    hasher.input(master);
    let mut master_hash = get_hash(&mut hasher);
    for i in 0..25_000_u32 {
        //Generate salt
        hasher.input(SALT);
        hasher.input(&i.to_le_bytes()[..]);
        let salt = get_hash(&mut hasher);
        //Feed in previous hash, salt and master password to modify hash
        hasher.input(&salt[..]);
        hasher.input(&master_hash[..]);
        hasher.input(master);
        master_hash = get_hash(&mut hasher);
    }
    //Hash both master and domain a few thousand times
    hasher.input(SALT);
    hasher.input(domain);
    hasher.input(master);
    let mut final_hash = get_hash(&mut hasher);
    for i in 0..25_000_u32 {
        //Generate salt for this iteration
        hasher.input(&i.to_be_bytes());
        hasher.input(SALT);
        let salt = get_hash(&mut hasher);
        //Feed in previous hash, salt and both master and domain in order to modify hash
        hasher.input(&salt[..]);
        hasher.input(&final_hash[..]);
        hasher.input(&master_hash[..]);
        hasher.input(master);
        hasher.input(domain);
        final_hash = get_hash(&mut hasher);
    }
    //Hopefully this algorithm is slow enough
    final_hash
}

/// The necessary configuration to generate a password.
///
/// Currently consisting of:
///
/// - A word list.
///
/// - An amount of words in the password.
///
/// - A style for joining the words together.
#[derive(Debug, Clone)]
pub struct Config<'a> {
    word_list: &'a WordList,
    word_count: u32,
    style: Style,
}
impl<'a> Config<'a> {
    pub fn new(word_list: &WordList) -> Config {
        Config {
            word_count: 0,
            style: Style::default(),
            word_list,
        }
        .with_entropy(DEFAULT_ENTROPY)
    }
    pub fn with_word_list<'b>(self, word_list: &'b WordList) -> Config<'b> {
        Config {
            word_list,
            word_count: self.word_count,
            style: self.style,
        }
    }
    pub fn word_list(&self) -> &'a WordList {
        self.word_list
    }
    pub fn with_word_count(mut self, word_count: u32) -> Self {
        self.word_count = word_count;
        self
    }
    pub fn word_count(&self) -> u32 {
        self.word_count
    }
    pub fn with_entropy(mut self, entropy: f64) -> Self {
        self.word_count = (entropy / (self.word_list.word_count() as f64).log2()).ceil() as u32;
        self
    }
    pub fn entropy(&self) -> f64 {
        (self.word_list.word_count() as f64).log2() * (self.word_count as f64)
    }
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn with_style_pascal(self) -> Self {
        self.with_style(Style::Pascal)
    }
    pub fn with_style_camel(self) -> Self {
        self.with_style(Style::Camel)
    }
    pub fn with_style_concat(self, separator: String) -> Self {
        self.with_style(Style::Concat(separator))
    }
    pub fn style(&self) -> &Style {
        &self.style
    }
    pub fn gen(&self, master: &[u8], domain: &[u8]) -> String {
        //Hash the master password and the domain together
        let hash = secure_hash(master, domain);
        //Use this hash to seed an RNG
        let mut rng = {
            const SEED_LEN: usize = 32;
            let mut seed = [0; SEED_LEN];
            seed.copy_from_slice(&hash[..SEED_LEN]);
            ChaChaRng::from_seed(seed)
        };
        //Now use the RNG to select words
        let mut password = String::new();
        for i in 0..self.word_count {
            let idx = rng.gen_range(0, self.word_list.word_count());
            let word = self.word_list.word(idx);
            self.style
                .push(word, &mut password, i == 0, i == self.word_count - 1);
        }
        password
    }
}
