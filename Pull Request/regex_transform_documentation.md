# Regex Transform Extension

The `regex_transform` extension performs regular expression search and replace operations on expansion variables.

## Parameters

* `source`: Input text. Supports variable injection.
* `find`: Search pattern.
* `replace`: Replacement template. Supports capture group backreferences like `$1`, `$2`, or `${1}`.
* `modifiers`: Map of capture group indices to casing transformations.
  * Options: `lowercase`, `uppercase`, `capitalize`.

## Double-Caps Autocorrect Example

Replace double capitalization (e.g., `BAnk` with `Bank`):

```yaml
matches:
  - regex: "\\b(?P<word>[A-Z]{2}[a-z]+)(?P<end>\\W)"
    replace: "{{corrected_word}}{{end}}"
    vars:
      - name: corrected_word
        type: regex_transform
        params:
          source: "{{word}}"
          find: "^([A-Z])([A-Z])(.*)$"
          replace: "$1$2$3"
          modifiers:
            "2": "lowercase"
```

Espanso captures the word, extracts the first two capitalized letters, downcases the second letter, and outputs the corrected word.
