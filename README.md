## Usage
```console
zw_to_anki -f ../Downloads/ST04.txt -o ST04.apkg
```
Then import the .apkg into Anki and you're good to go.

```console
Options:
  -f, --file <FILE>              File to be converted to flashcards
  -t, --text <TEXT>              Text to be converted to flashcards
  -o, --output <OUTPUT>          Output '.apkg' Anki deck path
      --hsk-filter <HSK_FILTER>  Optionally, an HSK level. Words that are in HSK at or below this level will not be added to the deck
  -h, --help                     Print help
  -V, --version                  Print version
```

## TODO:
 - add grabbing sound from google TTS and stick that on the flashcards
 - better heuristics about ignoring surnames, maybe?
 - squash duplicates into one note?
 - proper error handling, tests
 - allow to specify desired tone colours/turn them off
 - add examples from the text on each flashcard
 - CI/CD
