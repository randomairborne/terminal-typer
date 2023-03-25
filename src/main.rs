use std::{io::Write, time::Instant};

use crossterm::{
    cursor::{MoveTo, MoveToColumn, SetCursorStyle},
    event::{read, Event, KeyCode},
    style::{PrintStyledContent, Stylize},
    terminal::{self, ClearType, Clear}, ExecutableCommand, QueueableCommand, Result,
};
use rand::Rng;

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let to_type = get_text(args.length, args.file);
    let mut progress: u16 = 0;
    let chars: Vec<char> = to_type.chars().collect();
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(PrintStyledContent(to_type.on_grey()))?;
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(SetCursorStyle::BlinkingUnderScore)?;
    stdout.flush()?;
    let mut start: Option<Instant> = None;
    loop {
        let event = read()?;
        if let Event::Key(ke) = event {
            if let KeyCode::Char(c) = ke.code {
                let wanted_char = chars[progress as usize];
                if c == wanted_char {
                    if start.is_none() {
                        start = Some(Instant::now());
                    }
                    progress += 1;
                    stdout.queue(MoveTo(0, 0))?;
                    for i in 0..progress {
                        stdout.queue(PrintStyledContent(
                            chars[i as usize].yellow().on_dark_grey(),
                        ))?;
                    }
                    stdout.queue(MoveTo(progress, 0))?;
                    stdout.flush()?;
                }
                if chars.len() == progress as usize {
                    break;
                }
            }
        }
        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }
    let end = Instant::now();
    stdout.execute(MoveToColumn(0))?;
    terminal::disable_raw_mode()?;
    println!();
    let typed_so_far: String = chars[0..progress as usize].iter().collect();
    let total_words_typed = typed_so_far.split_whitespace().count();
    if let Some(start) = start {
        let total_time = end.duration_since(start);
        let wpm = total_words_typed as f64 / (total_time.as_secs_f64() / 60.0);
        println!(
            "Took {} seconds to type {} words ({} wpm)",
            total_time.as_secs(),
            total_words_typed,
            wpm.round()
        );
    }
    Ok(())
}

fn get_text(word_count: usize, file: Option<String>) -> String {
    let original = if let Some(file) = file {
        std::fs::read_to_string(file).expect("Filed did not exist")
    } else {
        include_str!("black_beauty.txt").to_string()
    };
    let mut book_as_one_sentence = String::with_capacity(original.len());
    let mut last_char = ' ';
    for char in original.chars() {
        if !(last_char.is_whitespace() && char.is_whitespace()) {
            if char.is_whitespace() {
                book_as_one_sentence.push(' ');
            } else if char == '“' || char == '”' {
                book_as_one_sentence.push('"')
            } else if char == '’' || char == '`' {
                book_as_one_sentence.push('\'')
            } else {
                book_as_one_sentence.push(char)
            }
        }
        last_char = char;
    }
    let words_in_book: Vec<&str> = book_as_one_sentence.split_whitespace().collect();
    let start = rand::thread_rng().gen_range(0..=(words_in_book.len() - word_count));
    let output_refs = &words_in_book[start..start + word_count];
    output_refs
        .iter()
        .copied()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(" ")
}

/// Test your typing speed, in the Terminal!
#[derive(argh::FromArgs)]
struct Args {
    /// length (in words) of text to type
    #[argh(option, short = 'l', default = "20")]
    length: usize,
    /// file to use instead of default (Black Beauty)
    #[argh(positional, short = 'f')]
    file: Option<String>,
}
