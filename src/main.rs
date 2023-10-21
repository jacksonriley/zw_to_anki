use clap::Parser;
use hsk::Hsk;
use jieba_rs::Jieba;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

mod anki;
mod dict;
mod pinyin;

use crate::anki::{Anki, ToneColours};
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

    /// Optionally: either "off" to turn tone colours off, or five semicolon-separated RGB colour codes for the five tones. For example, '00e304;b35815;f00f0f;1767fe;777777' (the default).
    #[arg(long)]
    tone_colours: Option<ToneColours>,
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
        let mut anki = Anki::new(
            o.split_once('.').unwrap().0,
            &args.tone_colours.unwrap_or_default(),
        );

        for word in words {
            if !cjk::is_simplified_chinese(word) {
                continue;
            }

            for result in dict.get(word) {
                // Optionally filter out words from lower HSK levels
                if let Some(hsk_filter) = args.hsk_filter {
                    let hsk_level = hsk_list.get_hsk(&result.simplified);
                    if hsk_level != 0 && hsk_level <= hsk_filter {
                        continue;
                    }
                }
                anki.add_note(result);
            }
        }

        anki.write_to_file(&o);
    }
}
