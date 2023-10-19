use crate::dict::Tone;

const VOWELS: &'static str = "aeiouüAEIOUÜ";

pub fn add_diacritic(text: &str, tone: Option<Tone>) -> String {
    let tone_which_needs_adding = match tone {
        None | Some(Tone::Fifth) => return text.into(),
        Some(other) => other,
    };

    // We have a tone, so it's regular pinyin (rather than being some english for e.g.)
    // If c is a consonant and v is a vowel, pinyin is always c*v+c*
    let first_vowel_idx = text
        .chars()
        .position(|c| VOWELS.contains(c))
        .expect("Pinyin always contains a vowel");
    let last_vowel_idx = text.len()
        - 1
        - text
            .chars()
            .rev()
            .position(|c| VOWELS.contains(c))
            .expect("Pinyin always contains a vowel");

    return format!(
        "{}{}{}",
        &text[..first_vowel_idx],
        add_diacritic_to_vowel_group(
            &text[first_vowel_idx..=last_vowel_idx],
            tone_which_needs_adding
        ),
        &text[last_vowel_idx + 1..]
    );
}

fn add_diacritic_to_vowel_group(vowels: &str, tone: Tone) -> String {
    // From https://en.wikipedia.org/wiki/Pinyin#Rules_for_placing_the_tone_mark
    if vowels.len() == 1 {
        return add_diacritic_to_char(vowels.chars().next().unwrap(), tone).to_string();
    } else if vowels.contains(|c| c == 'a' || c == 'e') {
        // If there is an a or an e, it will take the tone mark
        return vowels
            .chars()
            .map(|c| {
                if c == 'a' || c == 'e' {
                    add_diacritic_to_char(c, tone)
                } else {
                    c
                }
            })
            .collect();
    } else if vowels.contains("ou") {
        // If there is an ou, then the o takes the tone mark
        return vowels
            .chars()
            .map(|c| {
                if c == 'o' {
                    add_diacritic_to_char(c, tone)
                } else {
                    c
                }
            })
            .collect();
    } else {
        // Otherwise, the second vowel takes the tone mark
        return vowels
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 1 {
                    add_diacritic_to_char(c, tone)
                } else {
                    c
                }
            })
            .collect();
    }
}

const AS: [char; 5] = ['ā', 'á', 'ǎ', 'à', 'a'];
const ES: [char; 5] = ['ē', 'é', 'ě', 'è', 'e'];
const IS: [char; 5] = ['ī', 'í', 'ǐ', 'ì', 'i'];
const OS: [char; 5] = ['ō', 'ó', 'ǒ', 'ò', 'o'];
const US: [char; 5] = ['ū', 'ú', 'ǔ', 'ù', 'u'];
const UUS: [char; 5] = ['ǖ', 'ǘ', 'ǚ', 'ǜ', 'ü'];

fn add_diacritic_to_char(c: char, tone: Tone) -> char {
    let transformed = match c.to_lowercase().next().unwrap() {
        'a' => AS[usize::from(tone) - 1],
        'e' => ES[usize::from(tone) - 1],
        'i' => IS[usize::from(tone) - 1],
        'o' => OS[usize::from(tone) - 1],
        'u' => US[usize::from(tone) - 1],
        'ü' => UUS[usize::from(tone) - 1],
        _ => panic!("Invalid pinyin vowel: {c}"),
    };

    if c.is_uppercase() {
        transformed.to_uppercase().next().unwrap()
    } else {
        transformed
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_diacritics() {
        for (text, tone, expected) in [
            ("chu", Tone::Fourth, "chù"),
            ("liang", Tone::Second, "liáng"),
            ("Xi", Tone::First, "Xī"),
            ("Er", Tone::Third, "Ěr"),
            ("shuang", Tone::Fourth, "shuàng"),
        ] {
            assert_eq!(add_diacritic(text, Some(tone)), expected);
        }
    }
}
