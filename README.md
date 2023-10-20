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
 - probably better filtering of whitespace/punctuation
 - proper error handling, tests
 - allow to specify desired tone colours/turn them off
 - add examples from the text on each flashcard
 - add fallback for chunks not in the dictionary - fallback to adding characters. For e.g. jieba picks out "共同话题" as a chunk but this isn't in the dictionary. Can we be smarter than just adding individual chars? Perhaps greedily take from the front?:
   - :no: 共同话 && :yes: 题
   - :yes: 共同 && :yes: 话题 :green_tick:
 - CI/CD
