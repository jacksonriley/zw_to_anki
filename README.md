## Usage
```console
zw_to_anki -f ../Downloads/ST04.txt -o ST04.apkg
```
Then import the .apkg into Anki and you're good to go.

```console
Options:
  -f, --file <FILE>                  File to be converted to flashcards
  -t, --text <TEXT>                  Text to be converted to flashcards
  -o, --output <OUTPUT>              Output '.apkg' Anki deck path
      --hsk-filter <HSK_FILTER>      Optionally, an HSK level. Words that are in HSK at or below this level will not be added to the deck
      --tone-colours <TONE_COLOURS>  Optionally: either "off" to turn tone colours off, or five semicolon-separated RGB colour codes for the five tones. For example, '00e304;b35815;f00f0f;1767fe;777777' (the default)
  -s, --side <SIDE>                  Optionally: either 'ce-to-en' to produce only cards that test Chinese to English, or 'en-to-ce' for the opposite [possible values: ce-to-en, en-to-ce]
  -h, --help                         Print help
  -V, --version                      Print version
```

## TODO:
 - add grabbing sound from google TTS and stick that on the flashcards
 - better heuristics about filtering out HSK vocab. For example, with `--hsk-filter 2`, 帮助 is filtered out, but 帮 isn't. Probably need to just get the list and check if the word/character is contained within any HSK vocab.
 - better link to lookup the word when not on phone (i.e. not Pleco). Not sure what to use for this - maybe https://www.purpleculture.net/dictionary-details/?word=什么
 - better heuristics about ignoring surnames, maybe?
 - proper error handling, tests
 - add examples from the text on each flashcard
