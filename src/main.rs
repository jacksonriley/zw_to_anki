use clap::Parser;
use jieba_rs::Jieba;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;
use hsk::Hsk;

mod anki;
mod dict;
mod pinyin;

use crate::anki::Anki;
use crate::dict::CEDict;

/// Chunk up chinese text and make an Anki deck
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to be converted to flashcards
    #[arg(short, long)]
    file: Option<PathBuf>,
    
    /// Text to be converted to flashcards
    #[arg(short, long)]
    text: Option<String>,

    /// Output '.apkg' Anki deck path
    #[arg(short, long)]
    output: Option<String>,

    /// Optionally, an HSK level. Words that are in HSK at or below this level will not be added to the deck.
    #[arg(long)]
    hsk_filter: Option<u8>,
}

fn main() {
    // fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let to_chunk = match (args.file, args.text) {
        (Some(f), None) => read_to_string(f).unwrap(),
        (None, Some(t)) => t,
        _ => panic!("Supply either file or sentence"),
    };
    let jieba = Jieba::new();
    let words: HashSet<_> = jieba.cut(&to_chunk, false).into_iter().collect();

    println!("{:?}", &words);

	let hsk_list = Hsk::new();

    if let Some(o) = args.output {
        let dict = CEDict::new();
        let mut anki = Anki::new(o.split_once(".").unwrap().0);

        for word in words {
            if let Some(hsk_filter) = args.hsk_filter {
                let hsk_level = hsk_list.get_hsk(word);
                if hsk_level != 0 && hsk_level <= hsk_filter {
                    continue;
                }
            }

            if let Some(results) = dict.dict.get(word) {
                for result in results {
                    anki.add_note(result);
                }
            } else {
                eprintln!("No match for {word}");
            }
        }

        anki.write_to_file(&o);
    }
}
