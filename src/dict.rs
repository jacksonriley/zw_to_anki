use crate::pinyin::add_diacritic;
use std::collections::{BTreeSet, HashMap};
use std::convert::From;
use std::vec::Vec;

const CE_DICT: &str = include_str!("cedict_1_0_ts_utf-8_mdbg.txt");

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
}

impl From<u8> for Tone {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::First,
            2 => Self::Second,
            3 => Self::Third,
            4 => Self::Fourth,
            5 => Self::Fifth,
            other => panic!("Expected 1..5, got {other}"),
        }
    }
}

impl From<Tone> for usize {
    fn from(value: Tone) -> Self {
        match value {
            Tone::First => 1,
            Tone::Second => 2,
            Tone::Third => 3,
            Tone::Fourth => 4,
            Tone::Fifth => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinYin(pub Vec<PinYinSyllable>);

impl PinYin {
    pub fn colourise(&self) -> String {
        self.0
            .iter()
            .map(|pys| colourise(&add_diacritic(&pys.text, pys.tone), pys.tone))
            .collect::<String>()
    }
}

fn colourise(token: &str, tone: Option<Tone>) -> String {
    match tone {
        None => token.into(),
        Some(t) => format!(r#"<span class="tone{}">{}</span>"#, usize::from(t), token),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinYinSyllable {
    pub text: String,
    pub tone: Option<Tone>,
}

impl From<&str> for PinYinSyllable {
    fn from(value: &str) -> Self {
        // Parse from e.g.
        // 'yang3'
        // 'lu:4'
        if value == "·" {
            return Self {
                text: value.to_string(),
                tone: None,
            };
        }
        let (text, tone_str) = value.split_at(value.len() - 1);
        let tone = tone_str.parse::<u8>().ok().map(Tone::from);
        Self {
            // Note that ü is represented as 'u:' for some reason in the MDBG
            // dictionary, so fix that here.
            text: text.replace("u:", "ü"),
            tone,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    /// The simplified characters
    pub simplified: String,
    /// A mapping of pinyin reading to set of definitions for that reading
    pub pinyins: HashMap<PinYin, BTreeSet<String>>,
}

pub struct CEDict {
    pub dict: HashMap<String, Word>,
}

impl Default for CEDict {
    fn default() -> Self {
        Self::new()
    }
}
impl CEDict {
    pub fn new() -> Self {
        Self {
            dict: Self::parse(),
        }
    }

    fn parse() -> HashMap<String, Word> {
        let mut ret = HashMap::new();

        for line in CE_DICT.lines() {
            if line.starts_with('#') {
                continue;
            }
            let word = Self::parse_line(line);
            ret.entry(word.simplified.clone())
                .and_modify(|existing_word: &mut Word| {
                    for (py, defs) in &word.pinyins {
                        existing_word
                            .pinyins
                            .entry(py.clone())
                            .and_modify(|existing_defs| existing_defs.extend(defs.clone()))
                            .or_insert(defs.clone());
                    }
                })
                .or_insert(word);
        }

        ret
    }

    // Parse a line of the form
    // '一氧化氮 一氧化氮 [yi1 yang3 hua4 dan4] /nitric oxide/'
    fn parse_line(line: &str) -> Word {
        // Split first by / to get words and pinyin, and then all of the definitions.
        // Then split by [ to get the words and then the pinyin
        let mut defs = line.split('/');
        let words_and_pinyin = defs.next().unwrap();
        let (words, pinyin_trailing) = words_and_pinyin.split_once('[').unwrap();
        let simplified = words.split_whitespace().nth(1).unwrap();
        let pinyin = pinyin_trailing.trim_end_matches("] ");

        let pinyin = PinYin(
            pinyin
                .split_whitespace()
                .map(PinYinSyllable::from)
                .collect(),
        );

        let mut pinyins = HashMap::new();
        pinyins.insert(
            pinyin,
            defs.filter(|d| !d.is_empty()).map(String::from).collect(),
        );

        Word {
            simplified: simplified.to_string(),
            pinyins,
        }
    }

    /// Get all readings of a word.
    /// If the word is not in the dictionary, break it down to chunks and try
    /// to find the best chunking of the word that _is_ in the dictionary.
    /// For e.g., calling `get` with "共同话题" might return `Hanzi` for "共同"
    /// and "话题".
    pub fn get(&self, word: &str) -> Vec<&Word> {
        if let Some(results) = self.dict.get(word) {
            return vec![results];
        }

        // Need to segment to try to find chunks that _are_ in the dictionary.
        let chunkings = generate_all_chunkings(word);

        for chunking in chunkings {
            if let Some(results) = chunking
                .into_iter()
                .map(|chunk| self.dict.get(&chunk))
                .collect::<Option<Vec<_>>>()
            {
                // All of the chunks are in the dictionary!
                return results.into_iter().collect();
            }
        }

        panic!("The dictionary didn't contain one of the chars of {word}");
    }
}

/// Generate all chunkings of a word (save for the entire word)
/// For example, given "abc", will produce:
/// [
///     ["ab", "c"],
///     ["a", "bc"],
///     ["a", "b", "c"]
/// ]
fn generate_all_chunkings(word: &str) -> Vec<Vec<String>> {
    // TODO: It _feels_ like we should be able to return Vec<Vec<&str>> here
    // but I can't get it to compile
    let cs = word.chars().collect::<Vec<_>>();
    let mut results: Vec<Vec<String>> = Vec::new();
    iterate_all_subdivisions(&mut Vec::new(), &cs, &mut |x| {
        results.push(x.iter().map(|y| y.iter().collect::<String>()).collect());
    });
    results
        .into_iter()
        .rev() // Try the biggest chunking first
        .skip(1)
        .collect()
}

fn iterate_all_subdivisions<'a, F>(head: &mut Vec<&'a [char]>, rest: &'a [char], f: &mut F)
where
    F: FnMut(&[&[char]]),
{
    if rest.is_empty() {
        f(head);
    } else {
        for i in 1..=rest.len() {
            let (next, tail) = rest.split_at(i);
            head.push(next);
            iterate_all_subdivisions(head, tail, f);
            head.pop();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_line() {
        let word = CEDict::parse_line(
            "一氧化二氮 一氧化二氮 [yi1 yang3 hua4 er4 dan4] /nitrous oxide N2O/laughing gas/",
        );
        assert_eq!(word.simplified, "一氧化二氮");
        assert_eq!(
            word.pinyins
                .values()
                .flatten()
                .map(|s| &s[..])
                .collect::<Vec<&str>>(),
            vec!["laughing gas", "nitrous oxide N2O"]
        );
        assert_eq!(
            word.pinyins.keys().next().unwrap().0,
            vec![
                PinYinSyllable {
                    text: "yi".into(),
                    tone: Some(Tone::First)
                },
                PinYinSyllable {
                    text: "yang".into(),
                    tone: Some(Tone::Third)
                },
                PinYinSyllable {
                    text: "hua".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "er".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "dan".into(),
                    tone: Some(Tone::Fourth)
                }
            ]
        );
    }

    #[test]
    fn test_parse_name() {
        let word = CEDict::parse_line("亞歷山大·杜布切克 亚历山大·杜布切克 [Ya4 li4 shan1 da4 · Du4 bu4 qie1 ke4] /Alexander Dubček (1921-1992), leader of Czechoslovakia (1968-1969)/");
        assert_eq!(word.simplified, "亚历山大·杜布切克");
        assert_eq!(
            word.pinyins.keys().next().unwrap().0,
            vec![
                PinYinSyllable {
                    text: "Ya".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "li".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "shan".into(),
                    tone: Some(Tone::First)
                },
                PinYinSyllable {
                    text: "da".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "·".into(),
                    tone: None
                },
                PinYinSyllable {
                    text: "Du".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "bu".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYinSyllable {
                    text: "qie".into(),
                    tone: Some(Tone::First)
                },
                PinYinSyllable {
                    text: "ke".into(),
                    tone: Some(Tone::Fourth)
                }
            ]
        );
    }

    #[test]
    fn test_chunking() {
        assert_eq!(
            generate_all_chunkings("共同话题"),
            vec![
                vec!["共同话", "题"],
                vec!["共同", "话题"],
                vec!["共同", "话", "题"],
                vec!["共", "同话题"],
                vec!["共", "同话", "题"],
                vec!["共", "同", "话题"],
                vec!["共", "同", "话", "题"]
            ]
        );
    }
}
