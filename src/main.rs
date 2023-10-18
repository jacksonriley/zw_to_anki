use tts_rust::{languages::Languages, tts::GTTSClient};

use clap::Parser;
use jieba_rs::Jieba;
use std::fs::read_to_string;
use std::path::PathBuf;

mod anki;
mod dict;

use crate::anki::Anki;
use crate::dict::CEDict;

/// Chunk up chinese text and make an Anki deck
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to be converted to flashcards
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Output '.apkg' Anki deck path
    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    thing_to_say: Option<String>,
}

fn main() {
    // fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let to_chunk = match (args.file, args.thing_to_say) {
        (Some(f), None) => read_to_string(f).unwrap(),
        (None, Some(t)) => t,
        _ => panic!("Supply either file or sentence"),
    };
    let jieba = Jieba::new();
    let words = jieba.cut(&to_chunk, false);

    println!("{:?}", &words);

    if let Some(o) = args.output {
        let dict = CEDict::new();
        let mut anki = Anki::new(o.split_once(".").unwrap().0);

        for word in words {
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
