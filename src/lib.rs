

pub mod wordle {
    use std::{collections::{HashMap, HashSet}, fmt::Error, fs};
    use dialoguer::Input;
    use ansi_term::Color::{Black,Yellow,Green,White,Red};
    use termion::{clear, cursor};
    use rand::Rng;

    pub struct Secret {
        secret: String,
        letter_counts: HashMap<char, usize>,
    }
    fn string_to_letter_counts(text: &String) -> HashMap<char, usize> {
        // creates a hash map with counts of each unique occurrence of a char
        let mut counts = HashMap::new();
        
        for letter in text.chars() {
            hashmap_upsert(&mut counts, &letter);
        }
        counts
    }
    fn hashmap_upsert<'a>(hashmap: &'a mut HashMap<char, usize>, letter: &char) -> &'a mut HashMap<char, usize> {
        hashmap.entry(*letter)
            .and_modify(|key| *key += 1)
            .or_insert(1);
        hashmap
    }

    pub struct Game {
        secret: Secret,
        feedback: Vec<[isize; 5]>,
        pub guesses: Vec<String>,
        limit: usize,
        pub valid_guesses: HashSet<String>,
        pub valid_answers: HashSet<String>,
    }

    impl Secret {
        pub fn new_with_set_secret(set_secret: &str) -> Secret {
            let sec_string = String::from(set_secret);
            let counts = string_to_letter_counts(&sec_string);
            Secret {secret: sec_string, letter_counts: counts}
        }
        pub fn check_guess(&self, guess: &String) -> [isize; 5] {
            let mut feedback = [0 as isize; 5];
            let secret_letters: Vec<char>= self.secret.chars().collect();
            let mut counts: HashMap<char, usize> = HashMap::new();

            //println!("letter counts {:?}", self.letter_counts);

            for (i, g_char) in guess.chars().enumerate() {
                if g_char == secret_letters[i] {
                    feedback[i] = 2;
                    hashmap_upsert(&mut counts, &g_char);
                } else if secret_letters.contains(&g_char) {
                    let gchar_count = self.letter_counts.get(&g_char)
                    .unwrap_or_else(|| &0);
                    //println!("guess letter: {g_char}, count: {gchar_count}");
                    //need to only mark partial match up to the number of occurences of letter in secret
                    //
                    let guess_count = counts.get(&g_char).unwrap_or_else(|| &0);
                    
                    if gchar_count > guess_count {
                        feedback[i] = 1;
                    } else {feedback[i] = 0;}
                    hashmap_upsert(&mut counts, &g_char);
                } else {
                    feedback[i] = 0;
                    hashmap_upsert(&mut counts, &g_char);
                }
            }
            feedback
        }
    }

    impl Game {

        fn load_answer_set(path: &str) -> HashSet<String> {
            
            let contents = fs::read_to_string(path)
                .expect("configured_file_should exist");
                    
            let words: HashSet<String> = HashSet::from_iter(contents.split("\n")
                .map(|x| String::from(x).to_ascii_uppercase()));
                words
        }

        pub fn new_game() -> Game {
            let feedback: Vec<[isize; 5]> = Vec::new();
            let guesses: Vec<String> = Vec::new();
            let limit = 6;
            
            let valid_answers = Self::load_answer_set("wordle-La.txt");
            
            let answer_vec = Vec::from_iter(valid_answers.iter());
            let answer_key = rand::random_range(..answer_vec.len());
            let secret = Secret::new_with_set_secret(answer_vec[answer_key]);

            
            let valid_guesses = Self::load_answer_set("wordle-Ta.txt");

            let valid_guesses = HashSet::from_iter(valid_guesses.
                union(&valid_answers).map(|x| String::from(x)));

            Game {secret, feedback, guesses, limit, valid_guesses, valid_answers}
        }

        pub fn take_turn(&mut self) -> String {
            
            let mut guess: String;
            let mut word: Result<String, Error>;
            let mut first = true;
            let mut message = "Guess a word";
            loop {
                guess = Input::<String>::new()
                    .with_prompt(message)
                    .interact_text()
                    .unwrap();
                print!("{}", cursor::Up(1));
                print!("{}", clear::CurrentLine);    
                if first {
                    first = false;
                    message = "Invalid Guess! Try again"
                }
                word = self.validate_guess(guess);
                match word {
                    Ok(_) => break,
                    Err(Error) => continue,
                }
 
            }
        word.expect("error shoudl be handled")
        }

        fn validate_guess(&self, guess: String) -> Result<String, Error> {
            let output = guess.trim().to_ascii_uppercase();
            match self.valid_guesses.contains(&output) {
                true => Ok(output),
                false => Err(Error),
            }
        }

        pub fn print_result(feedback: &[isize;5], guess: &String) -> () {
            let mut print_line = Vec::new();
            
            for (i, letter) in guess.chars().enumerate() {
                let display_letter = match feedback[i] {
                    0 => White.on(Black)
                        .paint(format!(" {} ", letter)),
                    1 => Black.on(Yellow)
                        .paint(format!(" {} ", letter)),
                    2 => Black.on(Green)
                        .paint(format!(" {} ", letter)),
                    _ => continue,
                };
                print_line.push(display_letter);
            }
            print!("\t\t\t");

            

            for p in print_line.iter() {
                print!("|{}", p);
                // can add sleep statement here
            }
            print!("|\n");
            println!("\t\t\t―――――――――――――――――――――");
            ()
        }
        fn win(&self, turn: i32) -> () {
            let win_style = ansi_term::Style::new().blink()
                .bold().underline()
                .on(Green)
                .fg(Black);

            let stat_style = ansi_term::Style::new().bold()
                .on(Green)
                .fg(Black);

            println!("\n\n\t\t\t{}", win_style.paint("You got it!"));
            println!("\t\t\t{}", stat_style.paint(
                format!("{} / {} turns", turn, self.limit)
            ));
            println!("\t\t~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }
        fn lose(&self) -> () {
            let lose_style = ansi_term::Style::new().blink()
                .bold().underline()
                .on(Red)
                .fg(Black);

            let stat_style = ansi_term::Style::new().bold()
                .on(Red)
                .fg(Black);
            println!("\n\n\t\t\t{}", lose_style.paint("Try again next time..."));
            println!("\t\t\t{}", stat_style.paint(
                format!("The word was '{}'", self.secret.secret)
            ));
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }
        pub fn play(&mut self) -> () {
            let title_style = ansi_term::Style::new().bold().italic().underline().on(Black);
            // start game
            println!("{}", title_style.paint("\t\t\tWORDLE\t\t\t"));
            print!("a game about guessing 5 letter words");
            print!("{}", ansi_term::Style::new().blink().paint("...\n"));
            println!("\n\n\t\t\t―――――――――――――――――――――");
            // loop taking turns until limit or w
            let mut feedback: [isize; 5] = [0, 0, 0, 0, 0];
            let mut turn: usize = 0;
            while feedback != [2, 2, 2, 2, 2] && turn < self.limit {
                let guess = self.take_turn();
                feedback = self.secret.check_guess(&guess);
                Game::print_result(&feedback, &guess);
                self.guesses.push(guess);
                self.feedback.push(feedback.clone());
                turn += 1;    
            }
            if feedback == [2, 2, 2, 2, 2] {
                self.win(turn as i32);
            } else {
                self.lose();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::wordle::{Game, Secret};

    #[test]
    fn test_checker() -> () {
        let sec = Secret::new_with_set_secret("RIVER");
        let guess = String::from("RAISE");
        let feedback = sec.check_guess(&guess);

        assert_eq!(feedback, [2, 0, 1, 0, 1]);

        let sec = Secret::new_with_set_secret("RIVER");
        let guess = String::from("REESE");
        let feedback = sec.check_guess(&guess);

        assert_eq!(feedback, [2, 1, 0, 0, 0]);
    }
    #[test]
    fn test_new_game() -> () {
        let game = Game::new_game();

        assert!(game.valid_answers.contains(&String::from("RIVER")));
        assert!(game.valid_guesses.contains(&String::from("RIVER")));
        ()
    }
}