use pazzfraze::{Config, WordList};
use std::{env, fs, time::Instant};

fn print_usage() {
    eprintln!(
        r#"
usage: pazzfraze [OPTIONS] <wordlist> <master> <domain>
    --pascal    -p      Use `PascalStyle`.
    --camel     -c      Use `camelStyle`.
    --concat    -s      Concatenate words with an optional separator string.
    --entropy   -e      Accepts a number of bits of entropy.
    --length    -l      Accepts a number of words to form the password.
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let arg_start = match args.len().checked_sub(3) {
        Some(s) => s,
        _ => {
            print_usage();
            return;
        }
    };
    let (wordlist, master, domain) = {
        let args = &args[arg_start..];
        (&args[0], &args[1], &args[2])
    };
    //Open wordlist
    let words = match fs::read_to_string(wordlist) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("failed to open wordlist at \"{}\": {}", wordlist, err);
            return;
        }
    };
    let words = match WordList::new(words) {
        Some(w) => w,
        None => {
            eprintln!("no words found in wordlist at \"{}\"", wordlist);
            return;
        }
    };
    //Create config
    let mut conf = Config::new(&words);
    let mut options = args[..arg_start].iter().peekable();
    while let Some(opt) = options.next() {
        conf = match &**opt {
            "--pascal" | "-p" => conf.with_style_pascal(),
            "--camel" | "-c" => conf.with_style_camel(),
            "--concat" | "-s" => {
                let sep = if options
                    .peek()
                    .map(|opt| !opt.starts_with('-'))
                    .unwrap_or(false)
                {
                    //Use a separator string
                    options.next().unwrap().to_string()
                } else {
                    "".to_string()
                };
                conf.with_style_concat(sep)
            }
            "--entropy" | "-e" => {
                let num = match options.next() {
                    Some(n) => n,
                    None => {
                        print_usage();
                        return;
                    }
                };
                let num: f64 = match num.trim().parse() {
                    Ok(n) => n,
                    Err(_err) => {
                        print_usage();
                        return;
                    }
                };
                conf.with_entropy(num)
            }
            "--length" | "-l" => {
                let num = match options.next() {
                    Some(n) => n,
                    None => {
                        print_usage();
                        return;
                    }
                };
                let num: u32 = match num.trim().parse() {
                    Ok(n) => n,
                    Err(_err) => {
                        print_usage();
                        return;
                    }
                };
                conf.with_word_count(num)
            }
            _ => {
                eprintln!("unknown option \"{}\"", opt);
                print_usage();
                return;
            }
        }
    }
    //Generate password
    let start = Instant::now();
    let password = conf.gen(master.as_bytes(), domain.as_bytes());
    let finish = Instant::now();
    let time_taken = finish - start;
    eprintln!(
        "Generated password with {} bits of entropy in {}ms:",
        (conf.entropy() * 10.0).floor() / 10.0,
        time_taken.as_millis()
    );
    println!("{}", password);
    eprintln!("Remember to clear the terminal afterwards");
}
