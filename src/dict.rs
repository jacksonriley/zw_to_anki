use std::collections::HashMap;
use std::convert::From;
use std::vec::Vec;

const CE_DICT: &str = include_str!("cedict_1_0_ts_utf-8_mdbg.txt");

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct PinYin {
    pub text: String,
    pub tone: Option<Tone>,
}

impl From<&str> for PinYin {
    fn from(value: &str) -> Self {
        // Parse from e.g. yang3
        let (text, tone_str) = value.split_at(value.len() - 1);
        let tone = tone_str.parse::<u8>().ok().map(Tone::from);
        Self {
            text: text.to_string(),
            tone,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hanzi {
    pub simplified: String,
    pub pinyin: Vec<PinYin>,
    pub definitions: Vec<String>,
}

pub struct CEDict {
    pub dict: HashMap<String, Vec<Hanzi>>,
}

impl CEDict {
    pub fn new() -> Self {
        return Self {
            dict: Self::parse(),
        };
    }

    fn parse() -> HashMap<String, Vec<Hanzi>> {
        let mut ret = HashMap::new();

        for line in CE_DICT.lines() {
            if line.starts_with("#") {
                continue;
            }
            let hz = Self::parse_line(line);
            ret.entry(hz.simplified.clone())
                .and_modify(|v: &mut Vec<Hanzi>| v.push(hz.clone()))
                .or_insert(vec![hz]);
        }

        ret
    }

    // Parse a line of the form
    // '一氧化氮 一氧化氮 [yi1 yang3 hua4 dan4] /nitric oxide/'
    fn parse_line(line: &str) -> Hanzi {
        // Split first by / to get words and pinyin, and then all of the definitions.
        // Then split by [ to get the words and then the pinyin
        let mut defs = line.split("/");
        let words_and_pinyin = defs.next().unwrap();
        let (words, pinyin_trailing) = words_and_pinyin.split_once("[").unwrap();
        let simplified = words.split_whitespace().skip(1).next().unwrap();
        let pinyin = pinyin_trailing.trim_end_matches("] ");

        let pinyins = pinyin
            .split_whitespace()
            .filter(|py| *py != "·")
            .map(PinYin::from)
            .collect();

        Hanzi {
            simplified: simplified.to_string(),
            pinyin: pinyins,
            definitions: defs
                .filter(|d| !d.is_empty())
                .map(|d| d.to_string())
                .collect(),
        }
    }

    /// Get all readings of a word.
    /// If the word is not in the dictionary, break it down to chunks and try
    /// to find the best chunking of the word that _is_ in the dictionary.
    /// For e.g., calling `get` with "共同话题" might return `Hanzi` for "共同"
    /// and "话题".
    pub fn get(&self, word: &str) -> Vec<&Hanzi> {
        if let Some(results) = self.dict.get(word) {
            return results.iter().collect();
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
                return results.into_iter().flatten().collect();
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
        results.push(
            x.iter()
                .map(|y| y.into_iter().collect::<String>())
                .collect(),
        );
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
    if rest.len() == 0 {
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
        let hz = CEDict::parse_line(
            "一氧化二氮 一氧化二氮 [yi1 yang3 hua4 er4 dan4] /nitrous oxide N2O/laughing gas/",
        );
        assert_eq!(hz.simplified, "一氧化二氮");
        assert_eq!(hz.definitions, vec!["nitrous oxide N2O", "laughing gas"]);
        assert_eq!(
            hz.pinyin,
            vec![
                PinYin {
                    text: "yi".into(),
                    tone: Some(Tone::First)
                },
                PinYin {
                    text: "yang".into(),
                    tone: Some(Tone::Third)
                },
                PinYin {
                    text: "hua".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "er".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "dan".into(),
                    tone: Some(Tone::Fourth)
                }
            ]
        );
    }

    #[test]
    fn test_parse_name() {
        let hz = CEDict::parse_line("亞歷山大·杜布切克 亚历山大·杜布切克 [Ya4 li4 shan1 da4 · Du4 bu4 qie1 ke4] /Alexander Dubček (1921-1992), leader of Czechoslovakia (1968-1969)/");
        assert_eq!(hz.simplified, "亚历山大·杜布切克");
        assert_eq!(
            hz.pinyin,
            vec![
                PinYin {
                    text: "Ya".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "li".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "shan".into(),
                    tone: Some(Tone::First)
                },
                PinYin {
                    text: "da".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "Du".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "bu".into(),
                    tone: Some(Tone::Fourth)
                },
                PinYin {
                    text: "qie".into(),
                    tone: Some(Tone::First)
                },
                PinYin {
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
