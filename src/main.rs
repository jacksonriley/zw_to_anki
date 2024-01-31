use clap::Parser;
use futures::future;
use hsk::Hsk;
use jieba_rs::Jieba;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::PathBuf;

use zw_to_anki::anki::{Anki, Side, ToneColours};
use zw_to_anki::dict::CEDict;
use zw_to_anki::tts;

/// Chunk up chinese text and make an Anki deck
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to be converted to flashcards
    #[arg(short, long, conflicts_with = "text")]
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

    /// Optionally: either 'ce-to-en' to produce only cards that test Chinese to English, or
    /// 'en-to-ce' for the opposite.
    #[arg(value_enum, short, long)]
    side: Option<Side>,

    /// Add Chinese audio to each flashcard
    #[arg(long)]
    tts: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let to_chunk = match (args.file, args.text) {
        (Some(f), None) => read_to_string(f).unwrap(),
        (None, Some(t)) => t,
        _ => panic!("Supply either file or sentence"),
    };
    let mut jieba = Jieba::new();
    let dict = CEDict::new();
    for word in dict.dict.keys() {
        // Add in words from the MDBG set - we don't have frequency data for these but it still
        // seems likely that jieba having a larger vocabulary will help it to correctly segmentise.
        jieba.add_word(word, None, None);
    }
    let words: HashSet<_> = jieba.cut(&to_chunk, false).into_iter().collect();

    let hsk_list = Hsk::new();

    if let Some(o) = args.output {
        let mut words_for_cards = HashMap::new();

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

                // Don't create multiple cards with the same 汉字.
                if words_for_cards.contains_key(&result.simplified) {
                    continue;
                }

                words_for_cards.insert(result.simplified.clone(), result);
            }
        }

        let words_for_cards: Vec<_> = words_for_cards.into_values().collect();

        let mut anki = Anki::new(
            o.split_once('.').unwrap().0,
            &args.tone_colours.unwrap_or_default(),
            &args.side,
            args.tts,
        );

        let mut filenames = None;
        if args.tts {
            let client = reqwest::Client::new();
            let tts_futures = words_for_cards.iter().map(|word| {
                tts::save_to_file(
                    &client,
                    &word.simplified,
                    format!("mp3s/{}.mp3", word.simplified),
                )
            });
            filenames = Some(
                future::join_all(tts_futures)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            );
        }

        if let Some(ref fs) = filenames {
            for (word, filename) in words_for_cards.iter().zip(fs.iter()) {
                anki.add_note(word, Some(filename.strip_prefix("mp3s/").unwrap()));
            }
        } else {
            for word in &words_for_cards {
                anki.add_note(word, None);
            }
        }

        anki.write_to_file(
            &o,
            filenames.unwrap_or_default().iter().map(|s| &**s).collect(),
        );

        println!(
            "Successfully created a deck with {} notes",
            words_for_cards.len()
        );
    }
}
