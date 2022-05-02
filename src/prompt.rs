use anyhow::{Result, Context};
use std::io::{self, Write, stdin};

pub fn get_answer(prompt: &str, default: bool) -> Result<bool> {
    let switch;
    if default { switch = "(Y/n)"; } else { switch = "(y/N)"; }
    
    let answer_bool;
    loop {
        print!("{prompt} {switch}: ");
        io::stdout().flush().context("Unable to flush stdout")?;

        let mut answer = String::new();
        stdin().read_line(&mut answer).context("Unable to read from stdin")?;

        let answer_lowercase = answer.to_lowercase();
        let first_char = answer_lowercase.chars().next().context("Unable to get first character of your answer")?;
        answer_bool = match first_char {
            '\n' => default,
            '\r' => default,
            'y' => true,
            'n' => false,
            _ => {
                println!("Incorrect value submitted, please try again");
                continue; 
            },
        };
        break;
    }
    Ok(answer_bool)
}