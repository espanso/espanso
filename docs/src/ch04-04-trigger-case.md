# Trigger case sensitivity

Literal matches using `trigger` or `triggers` are case-sensitive by default. Set
`case_sensitive: false` to accept any capitalization while keeping the replacement
text's original capitalization:

```yaml
matches:
  - trigger: "brb/"
    replace: "I'll be right back"
    case_sensitive: false
```

Typing `brb/`, `Brb/`, `BRb/` or `BRB/` produces `I'll be right back`.
`propagate_case` defaults to `false`, so it does not need to be specified here.
The same setting applies to every entry in a `triggers` list and can be combined
with `word`, `left_word` and `right_word`.

## Matching and replacement capitalization

`case_sensitive` controls which typed triggers match. `propagate_case` controls
whether the replacement's capitalization follows the typed trigger. The options
are independent when `case_sensitive` is specified:

| case_sensitive | propagate_case | Trigger matching | Replacement capitalization |
| --- | --- | --- | --- |
| Omitted | Omitted or false | Case-sensitive | Unchanged |
| Omitted | true | Case-insensitive | Follows the typed trigger |
| false | Omitted or false | Case-insensitive | Unchanged |
| false | true | Case-insensitive | Follows the typed trigger |
| true | Either | Case-sensitive | Determined by propagate_case |

Omitting `case_sensitive` preserves the existing behavior of `propagate_case`.
For example, a match with only `propagate_case: true` continues to match any
capitalization and propagate the typed case to its output. An explicit
`case_sensitive: true` restricts matching to the stored trigger's case even when
case propagation is enabled.

This is a per-match setting for literal triggers, not a global configuration
option. It does not change regular expression matching. Use regex flags such as
`(?i)` to control case sensitivity in a `regex` match.
