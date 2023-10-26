use std::{collections::HashSet, str::FromStr};

use crate::dict::{Tone, Word};

use genanki_rs::{Deck, Field, Model, Note, Template};

#[derive(Debug, Clone)]
pub enum ToneColours {
    /// No tone colours
    Off,
    /// Specify semicolon-separated RGB codes for tones 1-5
    On([String; 5]),
}

impl FromStr for ToneColours {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "off" | "none" => Ok(ToneColours::Off),
            other => Ok(ToneColours::On(
                other
                    .split(';')
                    .map(|v| {
                        if v.len() == 6 && v.chars().all(char::is_alphanumeric) {
                            Ok(v.to_string())
                        } else {
                            Err(format!("Expected 6-char alphanumeric code, got {v}"))
                        }
                    })
                    .collect::<Result<Vec<String>, String>>()?
                    .try_into()
                    .map_err(|_| {
                        format!("Should specify five RGB codes, specified '{s}' instead")
                    })?,
            )),
        }
    }
}

impl Default for ToneColours {
    fn default() -> Self {
        ToneColours::On([
            "00e304".into(),
            "b35815".into(),
            "f00f0f".into(),
            "1767fe".into(),
            "777777".into(),
        ])
    }
}

impl ToneColours {
    fn css(&self) -> String {
        match self {
            ToneColours::Off => ".tone1 {color: black;}
                 .tone2 {color: black;}
                 .tone3 {color: black;}
                 .tone4 {color: black;}
                 .tone5 {color: black;}"
                .into(), // TODO: Feel like I should just be able to return String::default here... but then they end up blue (from the Pleco link I guess). What's going on?
            ToneColours::On([t1, t2, t3, t4, t5]) => format!(
                ".tone1 {{color: #{t1};}}
                 .tone2 {{color: #{t2};}}
                 .tone3 {{color: #{t3};}}
                 .tone4 {{color: #{t4};}}
                 .tone5 {{color: #{t5};}}"
            ),
        }
    }
}

pub struct Anki {
    model: Model,
    deck: Deck,
}

impl Anki {
    pub fn new(deck_name: &str, tone_colours: &ToneColours) -> Self {
        let model: Model = Model::new(
            1607392319,
            "Simple Model",
            vec![
                Field::new("AllDefinitions"),
                Field::new("AllDefinitionsWithPinyin"),
                Field::new("Hanzi"),
                Field::new("ColourHanzi"),
                Field::new("Example"),
            ],
            vec![
                Template::new("Card 1")
                    .qfmt("<div>{{AllDefinitions}}</div>")
                    .afmt(
                        r#"
                        <div class=chinese>
                            <a href="plecoapi://x-callback-url/s?q={{Hanzi}}" style="text-decoration:none">
                                {{ColourHanzi}}
                            </a>
                        </div>
                        <div>{{AllDefinitionsWithPinyin}}</div>
                        <div class=chinese>{{Example}}</div>
                        "#,
                    ),
                Template::new("Card 2")
                    .qfmt("<div class=chinese>{{Hanzi}}</div>")
                    .afmt(
                        r#"
                        <div class=chinese>
                            <a href="plecoapi://x-callback-url/s?q={{Hanzi}}" style="text-decoration:none">
                                {{ColourHanzi}}
                            </a>
                        </div>
                        <div>{{AllDefinitionsWithPinyin}}</div>
                        <div class=chinese>{{Example}}</div>
                        "#,
                    ),
            ],
        )
        .css(
            r#".card {
            font-family: arial;
            font-size: 20px;
            text-align: center;
            color: black;
            background-color: white;
        }
        .card { word-wrap: break-word; }
        .win .chinese { font-family: "MS Mincho", "ＭＳ 明朝"; }
        .mac .chinese { }
        .linux .chinese { font-family: "Kochi Mincho", "東風明朝"; }
        .mobile .chinese { font-family: "PingFang SC"; }
        .chinese { font-size: 30px;}
        
        .reading { font-size: 16px;}
        .comment {font-size: 15px; color:grey;}
        .tags {color:gray;text-align:right;font-size:10pt;}
        .note {color:gray;font-size:12pt;margin-top:20pt;}
        .hint {font-size:12pt;}
        .answer { background-color:bisque; border:dotted;border-width:1px}"#.to_string() + &tone_colours.css(),
        );

        let deck = Deck::new(1234, deck_name, "");

        Anki { model, deck }
    }

    pub fn add_note(&mut self, word: &Word) {
        // assert_eq!(word.simplified.chars().count(), word.pinyin.len());
        self.deck.add_note(
            Note::new(
                self.model.clone(),
                vec![
                    // AllDefinitions
                    &Self::to_all_definitions(word),
                    // AllDefinitionsWithPinyin
                    &Self::to_all_definitions_with_pinyin(word),
                    // Hanzi
                    &word.simplified,
                    // ColourHanzi
                    &Self::to_colour_hanzi(word),
                    // Pinyin
                    // Example
                    "",
                ],
            )
            .unwrap(),
        );
    }

    fn colourise(token: &str, tone: Option<Tone>) -> String {
        match tone {
            None => token.into(),
            Some(t) => format!(r#"<span class="tone{}">{}</span>"#, usize::from(t), token),
        }
    }

    pub fn write_to_file(&self, file: &str) {
        self.deck.write_to_file(file).unwrap()
    }

    fn to_all_definitions(word: &Word) -> String {
        word.pinyins
            .values()
            .map(|defs| {
                format!(
                    "<div>{}</div>",
                    defs.iter().cloned().collect::<Vec<_>>().join(" · ")
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn to_all_definitions_with_pinyin(word: &Word) -> String {
        word.pinyins
            .iter()
            .map(|(py, defs)| {
                format!(
                    "<div class=reading>{}</div><div>{}</div>",
                    py.colourise(),
                    defs.iter().cloned().collect::<Vec<_>>().join(" · ")
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn to_colour_hanzi(word: &Word) -> String {
        let tones_consensus = word
            .pinyins
            .keys()
            .map(|py| py.0.iter().map(|pys| pys.tone).collect::<Vec<_>>())
            .collect::<HashSet<_>>();

        if word.simplified == "干" {
            eprintln!("{:?}", word);
            eprintln!("{:?}", tones_consensus);
        }

        if tones_consensus.len() == 1 {
            // There may or may not be multiple readings of this word, but they
            // all have the same tones, so use that
            word.simplified
                .chars()
                .zip(tones_consensus.into_iter().next().unwrap())
                .map(|(c, t)| Self::colourise(&c.to_string(), t))
                .collect::<String>()
        } else {
            // There are multiple tone patterns for this word, just return as is
            word.simplified
                .chars()
                .map(|c| Self::colourise(&c.to_string(), Some(Tone::Fifth)))
                .collect::<String>()
        }
    }
}
