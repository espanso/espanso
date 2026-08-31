/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::de::DeserializeOwned;

/// Check if the given string represents an empty YAML.
/// In other words, it checks if the document is only composed
/// of spaces and/or comments
pub fn is_yaml_empty(yaml: &str) -> bool {
    for line in yaml.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with('#') && !trimmed_line.is_empty() {
            return false;
        }
    }

    true
}

/// The first characters of a *bare* (unquoted) flow-collection value that this module
/// re-quotes, restoring the pre-2.4.0 `serde_yaml` 0.8 leniency that PR #2532
/// (`serde_yaml` -> `serde_norway`, which is libyaml-based and strict) removed.
///
/// The reported regression (#2748) is values like `:->`, `:>-`, `:-)`, `:>hello` —
/// emoticons and arrows whose first character is `:` or `>` (and the block indicators
/// `|`/`>` used as literal text, e.g. a `|`/`>`-led value). libyaml forbids these as a
/// plain scalar's first character inside a flow collection, so it rejects them with
/// *"did not find expected node content ... while parsing a flow node"*.
///
/// This set is deliberately NARROW. libyaml also forbids `*`, `!`, `&` (alias / tag /
/// anchor indicators), `@`, `%`, the backtick (reserved), and `#` (comment) — but those
/// carry their own YAML meaning, and pre-2.4.0 did *not* hand them back as plain strings
/// either, so accepting them would widen YAML rather than restore the old behaviour.
/// They are intentionally left strict: if a config fails strict parsing because of one of
/// them, the retry still fails and the original error is returned unchanged.
const LENIENT_FLOW_VALUE_FIRST_CHARS: &[char] = &[':', '>', '|'];

/// Parse `yaml` strictly first; only if that fails, retry once on a leniently
/// re-quoted copy (see [`lenient_flow_quote`]).
///
/// This restores the pre-2.4.0 `serde_yaml` tolerance for unquoted flow values that
/// start with a YAML indicator character (e.g. `triggers: [:->,:>-]`), which PR #2532
/// (`serde_yaml` -> `serde_norway`) inadvertently broke.
///
/// The retry-on-failure gate is the central safety property: because the lenient
/// transform only ever runs on input the strict parser has *already rejected*, every
/// configuration that parses today is returned byte-for-byte untouched. This cannot
/// change the behaviour of any valid configuration. If the retry still fails, the
/// *original* error is returned, so genuinely malformed YAML fails exactly as before.
pub fn parse_lenient<T: DeserializeOwned>(yaml: &str) -> Result<T, serde_norway::Error> {
    match serde_norway::from_str(yaml) {
        Ok(value) => Ok(value),
        Err(original) => match serde_norway::from_str(&lenient_flow_quote(yaml)) {
            Ok(value) => Ok(value),
            Err(_retry) => Err(original),
        },
    }
}

/// Re-quote bare flow-collection scalars whose first character is one of
/// [`LENIENT_FLOW_VALUE_FIRST_CHARS`] so that `serde_norway` accepts them.
///
/// This is a deliberately conservative single forward scan. It only ever touches
/// characters that appear *inside* a `[...]`/`{...}` collection (`depth > 0`), and
/// only when a bare value scalar begins with one of those characters. Already-quoted
/// scalars (`'...'` / `"..."`) and `#` comments are skipped verbatim, so they
/// round-trip unchanged. Block-context scalars (outside any flow collection) are never
/// modified.
#[must_use]
pub fn lenient_flow_quote(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(input.len());
    let mut depth: i32 = 0;
    // True at positions where a bare scalar could begin (after `[`, `{`, `,` in flow,
    // or after a `:` value indicator in flow). Only meaningful together with depth > 0.
    let mut scalar_start_pending = true;
    let mut i = 0;

    while i < n {
        // At a line start in block context, a `|`/`>` block-scalar header introduces a
        // literal/folded body whose contents must NEVER be transformed (the body is plain
        // text, not YAML structure, so its `[ ... ]` / `{ ... }` are not flow collections
        // and a bare `:`/`>` is not an indicator). Copy the whole block (header + body)
        // verbatim and resume after it. See [`block_scalar_span`].
        if (i == 0 || chars[i - 1] == '\n') && depth == 0 {
            if let Some(resume) = block_scalar_span(&chars, i) {
                out.extend(&chars[i..resume]);
                i = resume;
                continue;
            }
        }

        let c = chars[i];

        // Already-quoted scalars are copied verbatim and never re-touched.
        if c == '\'' || c == '"' {
            i = copy_quoted_scalar(&chars, i, &mut out);
            scalar_start_pending = false;
            continue;
        }

        // Flow collection open: descend. A scalar may start right after it.
        if c == '[' || c == '{' {
            depth += 1;
            out.push(c);
            scalar_start_pending = true;
            i += 1;
            continue;
        }
        // Flow collection close: ascend.
        if c == ']' || c == '}' {
            depth = (depth - 1).max(0);
            out.push(c);
            scalar_start_pending = false;
            i += 1;
            continue;
        }

        // A `#` preceded by whitespace (or at a line start) begins a comment that runs
        // to the end of the line; copy it verbatim, regardless of depth.
        if c == '#' && comment_starts_here(&chars, i) {
            i = copy_comment(&chars, i, &mut out);
            continue;
        }

        if depth > 0 {
            // Flow separator: a new scalar may start after it.
            if c == ',' {
                out.push(c);
                scalar_start_pending = true;
                i += 1;
                continue;
            }
            // `:` is a key/value indicator only when followed by whitespace, a flow
            // terminator/separator, or end-of-input; then a value scalar follows.
            if c == ':' && is_value_indicator(&chars, i) {
                out.push(c);
                scalar_start_pending = true;
                i += 1;
                continue;
            }

            if scalar_start_pending {
                // Skip leading whitespace before the candidate scalar; stay pending.
                if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                    out.push(c);
                    i += 1;
                    continue;
                }
                if LENIENT_FLOW_VALUE_FIRST_CHARS.contains(&c) {
                    // Bare indicator-led scalar: single-quote it.
                    let (resume_at, value, trailing) = extract_bare_scalar(&chars, i);
                    out.push('\'');
                    for ec in value.chars() {
                        // Inside a single-quoted scalar, a literal `'` is escaped as `''`.
                        if ec == '\'' {
                            out.push('\'');
                            out.push('\'');
                        } else {
                            out.push(ec);
                        }
                    }
                    out.push('\'');
                    out.push_str(&trailing);
                    scalar_start_pending = false;
                    i = resume_at;
                    continue;
                }
                // The scalar starts with an ordinary character: emit it and stop being
                // pending. The remainder flows through the default verbatim path.
                out.push(c);
                scalar_start_pending = false;
                i += 1;
                continue;
            }
        }

        // Default: copy verbatim. Whitespace preserves the pending state; any other
        // character means we are no longer at a scalar-start position.
        out.push(c);
        if !(c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            scalar_start_pending = false;
        }
        i += 1;
    }

    out
}

/// Copy a `'...'` or `"..."` quoted scalar verbatim into `out`, handling `''` (single)
/// and `\` (double) escapes, and return the index just past the closing quote.
fn copy_quoted_scalar(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    let quote = chars[start];
    out.push(quote);
    let mut i = start + 1;
    if quote == '\'' {
        while i < n {
            let c = chars[i];
            out.push(c);
            i += 1;
            if c == '\'' {
                // `''` is an escaped single quote; keep consuming if doubled.
                if i < n && chars[i] == '\'' {
                    out.push('\'');
                    i += 1;
                } else {
                    break;
                }
            }
        }
    } else {
        while i < n {
            let c = chars[i];
            out.push(c);
            i += 1;
            if c == '\\' {
                // Escaped character inside a double-quoted scalar: copy it verbatim.
                if i < n {
                    out.push(chars[i]);
                    i += 1;
                }
            } else if c == '"' {
                break;
            }
        }
    }
    i
}

/// Copy a `#` comment verbatim into `out`, up to (but excluding) the next newline,
/// and return the index of that newline (or end-of-input).
fn copy_comment(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    let mut i = start;
    while i < n && chars[i] != '\n' {
        out.push(chars[i]);
        i += 1;
    }
    i
}

/// Whether the `:` at `chars[i]` acts as a key/value indicator: it must be followed by
/// whitespace, a flow separator/terminator, or end-of-input.
fn is_value_indicator(chars: &[char], i: usize) -> bool {
    debug_assert_eq!(chars.get(i), Some(&':'));
    match chars.get(i + 1) {
        None => true,
        Some(next) => matches!(next, ' ' | '\t' | '\n' | '\r' | ',' | ']' | '}'),
    }
}

/// Whether the `#` at `chars[i]` begins a comment: it is at a line start or preceded
/// by whitespace.
fn comment_starts_here(chars: &[char], i: usize) -> bool {
    debug_assert_eq!(chars.get(i), Some(&'#'));
    if i == 0 {
        return true;
    }
    matches!(chars[i - 1], ' ' | '\t' | '\n' | '\r')
}

/// From a bare-scalar start at `start`, extract the scalar value (trailing whitespace
/// trimmed) and the trailing whitespace that followed it, and return the index of the
/// terminator the caller should resume scanning from. The terminator is the first of
/// `,`, `[`, `]`, `{`, `}`, a newline, a `#` comment, or a `:` that acts as a key/value
/// indicator — because a plain (unquoted) flow scalar can never legitimately contain a
/// `:` followed by whitespace, that `:` always ends the scalar (e.g. it ends a `:`-led
/// *key* in `{ :k: v }`).
fn extract_bare_scalar(chars: &[char], start: usize) -> (usize, String, String) {
    let n = chars.len();
    let mut j = start;
    while j < n {
        let c = chars[j];
        if matches!(c, ',' | '[' | ']' | '{' | '}' | '\n' | '\r') {
            break;
        }
        if c == '#' && comment_starts_here(chars, j) {
            break;
        }
        if c == ':' && is_value_indicator(chars, j) {
            break;
        }
        j += 1;
    }
    let raw = &chars[start..j];
    let mut end = raw.len();
    while end > 0 && matches!(raw[end - 1], ' ' | '\t') {
        end -= 1;
    }
    let value: String = raw[..end].iter().collect();
    let trailing: String = raw[end..].iter().collect();
    (j, value, trailing)
}

/// If the line starting at `line_start` (a char index right after a newline, or 0) is a
/// block-scalar header in block context, return the char index at which to resume
/// scanning — i.e. the start of the first line that is *not* part of the block body
/// (the line that de-dents back to the header's indentation), or end-of-input if the
/// body runs to the end. Returns `None` if the line is not a block-scalar header.
///
/// The header line and its entire body are meant to be copied verbatim, so that literal
/// text in a `|`/`>` block scalar (shell snippets, code, regex, examples containing
/// `[ ... ]` / `{ ... }` / `:` / `>`) is never mistaken for YAML flow structure.
///
/// Detection is deliberately **liberal** (it errs toward treating an ambiguous line as a
/// header): a false positive only copies some lines verbatim, which can miss a fix but
/// can never corrupt data, whereas a false negative could let a body be re-quoted.
fn block_scalar_span(chars: &[char], line_start: usize) -> Option<usize> {
    let n = chars.len();
    // Leading whitespace = the header key's (parent) indentation.
    let mut pos = line_start;
    let mut parent_indent = 0usize;
    while pos < n && (chars[pos] == ' ' || chars[pos] == '\t') {
        parent_indent += 1;
        pos += 1;
    }
    if !is_block_scalar_header(chars, pos) {
        return None;
    }
    // Advance to the newline ending the header line (or end-of-input).
    let mut cur = pos;
    while cur < n && chars[cur] != '\n' {
        cur += 1;
    }
    // Walk the body line by line. The body is subsequent blank lines plus lines whose
    // indentation is strictly greater than the header's. It ends at the first non-blank
    // line indented at most `parent_indent` (or at end-of-input).
    loop {
        if cur >= n {
            break;
        }
        // Consume the newline to reach the next line's first character.
        cur += 1;
        if cur >= n {
            break;
        }
        // Indentation of this line.
        let mut content = cur;
        let mut line_indent = 0usize;
        while content < n && (chars[content] == ' ' || chars[content] == '\t') {
            line_indent += 1;
            content += 1;
        }
        // Blank line (only whitespace up to the newline)? Part of the body / its trailer.
        let mut scan = content;
        let mut blank = true;
        while scan < n && chars[scan] != '\n' {
            if !matches!(chars[scan], ' ' | '\t' | '\r') {
                blank = false;
                break;
            }
            scan += 1;
        }
        if blank {
            // Keep the line; advance to its newline.
            while cur < n && chars[cur] != '\n' {
                cur += 1;
            }
            continue;
        }
        if line_indent > parent_indent {
            // Body line; advance to its newline.
            while cur < n && chars[cur] != '\n' {
                cur += 1;
            }
            continue;
        }
        // De-dent: this line ends the block. `cur` is already at its first character.
        break;
    }
    Some(cur)
}

/// Whether the line whose first non-whitespace character is at `start` is a YAML
/// block-scalar header.
///
/// A block header's `|` or `>` is always the value (after a `:`, a `-`, an optional tag
/// `!…` and/or anchor `&…`), followed only by an optional chomping/indentation indicator
/// (`+`/`-`/digits), optional whitespace, and an optional `#` comment. The detector
/// therefore accepts any `|`/`>` with exactly that suffix and intentionally does NOT
/// inspect what precedes the indicator: requiring a `:`/`-` immediately before it would
/// false-negative on anchors (`a: &x |`) and tags (`a: !t |`), leaving the body
/// unprotected and corrupting it. A false positive here (a plain scalar that happens to
/// end in `|`/`>`) only causes some lines to be copied verbatim — a missed fix, never
/// corruption — which is the safe direction.
fn is_block_scalar_header(chars: &[char], start: usize) -> bool {
    let n = chars.len();
    // End of the line (index of '\n' or n).
    let mut eol = start;
    while eol < n && chars[eol] != '\n' {
        eol += 1;
    }
    // The block indicator is the last `|` or `>` on the line.
    let mut indicator = None;
    for idx in (start..eol).rev() {
        if chars[idx] == '|' || chars[idx] == '>' {
            indicator = Some(idx);
            break;
        }
    }
    let Some(indicator) = indicator else {
        return false;
    };
    // After the indicator: optional chomping/indent indicator chars, then optional
    // whitespace, then either end-of-line or a `#` comment. Anything else means the
    // `|`/`>` is plain text (e.g. `a>b`, `"|"`, `| body`), not a block header.
    let mut after = indicator + 1;
    while after < eol
        && (chars[after] == '+' || chars[after] == '-' || chars[after].is_ascii_digit())
    {
        after += 1;
    }
    while after < eol && matches!(chars[after], ' ' | '\t') {
        after += 1;
    }
    if after < eol && chars[after] == '#' {
        return true;
    }
    after == eol
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::{fs::create_dir_all, path::Path};
    use tempdir::TempDir;

    pub fn use_test_directory(callback: impl FnOnce(&Path, &Path, &Path)) {
        let dir = TempDir::new("tempconfig").unwrap();
        let match_dir = dir.path().join("match");
        create_dir_all(&match_dir).unwrap();

        let config_dir = dir.path().join("config");
        create_dir_all(&config_dir).unwrap();

        callback(
            &dunce::canonicalize(dir.path()).unwrap(),
            &dunce::canonicalize(match_dir).unwrap(),
            &dunce::canonicalize(config_dir).unwrap(),
        );
    }

    #[test]
    fn is_yaml_empty_document_empty() {
        assert!(is_yaml_empty(""));
    }

    #[test]
    fn is_yaml_empty_document_with_comments() {
        assert!(is_yaml_empty("\n#comment \n \n"));
    }

    #[test]
    fn is_yaml_empty_document_with_comments_and_content() {
        assert!(!is_yaml_empty("\n#comment \n field: true\n"));
    }

    #[test]
    fn is_yaml_empty_document_with_content() {
        assert!(!is_yaml_empty("\nfield: true\n"));
    }

    // --- lenient_flow_quote: transformation behaviour ---

    #[test]
    fn lenient_flow_quote_identity_without_flow_collections() {
        // No flow collection anywhere: output must be byte-identical.
        let inputs = [
            "hello: world",
            "# just a comment\n",
            "trigger: \":->\"\nreplace: smile\n",
            "matches:\n  - trigger: hello\n    replace: hi\n",
        ];
        for input in inputs {
            assert_eq!(
                lenient_flow_quote(input),
                input,
                "non-flow input was modified: {input:?}"
            );
        }
    }

    #[test]
    fn lenient_flow_quote_quotes_issue_2748_cases() {
        assert_eq!(lenient_flow_quote("[:->,:>-]"), "[':->',':>-']");
        assert_eq!(lenient_flow_quote("[:-), :-D]"), "[':-)', ':-D']");
        // `-` is NOT a flow indicator: `->test` stays bare, only `:>hello` is quoted.
        assert_eq!(
            lenient_flow_quote("[->test, :>hello]"),
            "[->test, ':>hello']"
        );
        // Flow-mapping value.
        assert_eq!(lenient_flow_quote("{a: :x}"), "{a: ':x'}");
        // Flow-mapping KEY that starts with `:`: only the key up to its `:` separator is
        // quoted, so the entry stays a key/value pair (not collapsed into one key).
        assert_eq!(lenient_flow_quote("{:k: v}"), "{':k': v}");
        assert_eq!(lenient_flow_quote("{:k: v, :m: n}"), "{':k': v, ':m': n}");
        // A value containing `:` not followed by whitespace stays one scalar.
        assert_eq!(lenient_flow_quote("[:a:b]"), "[':a:b']");
        // Nested depth: the inner `:>x` is quoted, the nesting is preserved.
        assert_eq!(lenient_flow_quote("[[:>x]]"), "[[':>x']]");
        // Comment after the value; trailing spaces kept outside the quotes.
        assert_eq!(lenient_flow_quote("[ :x  # c\n ]"), "[ ':x'  # c\n ]");
    }

    // A `|`/`>` block-scalar header introduces a literal/folded body that is plain text,
    // not YAML structure. Its body must be copied byte-for-byte even when it contains
    // flow-like text (`[ :>x ]`, `{ ... }`, bare `:`/`>`).
    #[test]
    fn lenient_flow_quote_preserves_block_scalar_bodies() {
        // Literal block scalar whose body contains `[ :>x ]`: the trigger is fixed but
        // the body is byte-identical.
        let input = "triggers: [:-)]\nreplace: |\n  if [ :>x ]; then echo hi; fi\n";
        assert_eq!(
            lenient_flow_quote(input),
            "triggers: [':-)']\nreplace: |\n  if [ :>x ]; then echo hi; fi\n"
        );

        // Chomping indicator `|-` and a body with braces/colons.
        let input = "x: |-\n  map {a: :b}\n  arrow :>z\n";
        assert_eq!(lenient_flow_quote(input), input);

        // Folded scalar `>`.
        let input = "x: >\n  folded [ :>y ] text\n";
        assert_eq!(lenient_flow_quote(input), input);

        // Sequence of block scalars.
        let input = "- |\n  body [ :>a ]\n- |\n  body [ :>b ]\n";
        assert_eq!(lenient_flow_quote(input), input);

        // A block scalar does NOT stop a later flow collection from being fixed.
        let input = "replace: |\n  [ :>body ]\ntriggers: [:-)]\n";
        assert_eq!(
            lenient_flow_quote(input),
            "replace: |\n  [ :>body ]\ntriggers: [':-)']\n"
        );

        // A `|`/`>` that is NOT a header (quoted, or with trailing content) is not
        // treated as a block scalar, so its line is scanned normally.
        assert_eq!(lenient_flow_quote("x: \"|\"\n"), "x: \"|\"\n");
        assert_eq!(lenient_flow_quote("x: a>b\n"), "x: a>b\n");
        assert_eq!(lenient_flow_quote("x: | body\n"), "x: | body\n");

        // A header with a trailing comment is still detected.
        let input = "replace: | # a header\n  body [ :>x ]\n";
        assert_eq!(lenient_flow_quote(input), input);

        // Anchors (`&x`), tags (`!t`) and a quoted key must NOT defeat header detection
        // (a false negative here would re-quote inside the body = corruption).
        let input = "a: &x |\n  [ :>y ]\ntrig: [:-)]\n";
        assert_eq!(
            lenient_flow_quote(input),
            "a: &x |\n  [ :>y ]\ntrig: [':-)']\n"
        );
        let input = "a: !t |\n  [ :>y ]\ntrig: [:-)]\n";
        assert_eq!(
            lenient_flow_quote(input),
            "a: !t |\n  [ :>y ]\ntrig: [':-)']\n"
        );
    }

    #[test]
    fn lenient_flow_quote_quotes_literal_value_first_chars() {
        // Each of these flow values starts with a literal-text indicator (`:`, `>`, `|`)
        // that libyaml forbids as a plain-scalar first character. After quoting,
        // serde_norway must accept them and recover the exact value.
        for (input, expected_value) in [
            ("[:val]", ":val"),
            ("[:->,:>-]", ":->"), // first element of the #2748 reproducer
            ("[>val]", ">val"),
            ("[|val]", "|val"),
        ] {
            let quoted = lenient_flow_quote(input);
            let parsed: serde_norway::Value = serde_norway::from_str(&quoted)
                .unwrap_or_else(|e| panic!("{input:?} -> {quoted:?}: {e}"));
            assert_eq!(
                parsed[0], expected_value,
                "value mismatch for {input:?} -> {quoted:?}"
            );
        }
    }

    #[test]
    fn lenient_flow_quote_does_not_touch_feature_or_reserved_indicators() {
        // YAML feature / reserved indicators (`*` alias, `!` tag, `&` anchor, `@` `%`
        // backtick reserved) are NOT re-quoted: pre-2.4.0 did not hand them back as plain
        // strings either, so accepting them would widen YAML rather than restore the old
        // behaviour. The helper leaves them byte-identical, and parse_lenient's outcome
        // matches the strict parser exactly for them (no new leniency is introduced).
        for input in ["[*val]", "[!val]", "[&val]", "[@val]", "[%val]", "[`val]"] {
            assert_eq!(
                lenient_flow_quote(input),
                input,
                "feature/reserved indicator was modified: {input:?}"
            );
            let strict: Result<serde_norway::Value, _> = serde_norway::from_str(input);
            let gated: Result<serde_norway::Value, _> = parse_lenient(input);
            assert_eq!(
                strict.is_ok(),
                gated.is_ok(),
                "parse_lenient must not change the outcome for {input:?}"
            );
        }
    }

    #[test]
    fn lenient_flow_quote_keeps_dash_led_scalar_bare() {
        // `-` is not a flow indicator: a value starting with `-` must not be quoted.
        let quoted = lenient_flow_quote("[-test]");
        assert_eq!(quoted, "[-test]");
        let parsed: serde_norway::Value = serde_norway::from_str(&quoted).unwrap();
        assert_eq!(parsed[0], "-test");
    }

    #[test]
    fn lenient_flow_quote_leaves_quoted_scalars_untouched() {
        for input in ["[':->']", "[\":->\"]", "['hello', \"john\"]"] {
            assert_eq!(lenient_flow_quote(input), input);
        }
    }

    #[test]
    fn lenient_flow_quote_escapes_embedded_single_quote() {
        // A bare value containing a `'` must escape it as `''` inside the quotes.
        let quoted = lenient_flow_quote("[:it's]");
        assert_eq!(quoted, "[':it''s']");
        let parsed: serde_norway::Value = serde_norway::from_str(&quoted).unwrap();
        assert_eq!(parsed[0], ":it's");
    }

    // --- lenient_flow_quote: zero-regression for already-valid YAML ---

    #[test]
    fn lenient_flow_quote_does_not_change_valid_flow_values() {
        // Each input parses strictly today; the helper must not change its parsed value.
        let inputs = [
            "triggers: [\"hello\", \"john\"]",
            "triggers: [hello, john]",
            "trigger: \":->\"",
            "triggers: [a, b, c]",
            "triggers: [->test, hello]",
            "mappings: {a: b, c: d}",
            "list: [1, 2, 3]",
        ];
        for input in inputs {
            let original: serde_norway::Value = serde_norway::from_str(input)
                .unwrap_or_else(|e| panic!("baseline should parse: {input:?}: {e}"));
            let lenient: serde_norway::Value = serde_norway::from_str(&lenient_flow_quote(input))
                .unwrap_or_else(|e| panic!("lenient should parse: {input:?}: {e}"));
            assert_eq!(
                original, lenient,
                "transform changed parsed value for: {input:?}"
            );
        }
    }

    #[test]
    fn parse_lenient_returns_strict_error_for_genuinely_malformed_yaml() {
        // An unbalanced flow collection is malformed for reasons the helper cannot fix:
        // the original error must be returned (not a wrong-but-valid parse).
        let result: Result<serde_norway::Value, _> = parse_lenient("[[:->");
        assert!(result.is_err());
    }

    #[test]
    fn parse_lenient_does_not_corrupt_pathological_flow_input() {
        // A pathological input the helper cannot cleanly repair must not turn into a
        // different-but-valid document: the original error is returned instead.
        let result: Result<serde_norway::Value, _> = parse_lenient("[:[:>x]]");
        assert!(result.is_err());
    }

    // A realistic corpus of espanso match/config snippets. For each VALID snippet the
    // strict parse must succeed and the helper must not change its parsed value; for each
    // #2748-style snippet (strictly rejected) parse_lenient must recover the expected value.
    #[test]
    fn lenient_flow_quote_corpus_zero_regression_and_fixes() {
        // (a) Already-valid YAML: helper must be value-preserving.
        let valid = [
            "trigger: ':hello'\nreplace: world\n",
            "triggers: [':->', ':>-']\n",
            "triggers: [hello, world, foo]\n",
            "triggers: [':->', \":>-\"]\n",
            "triggers: [a:b, c:d]\n",
            "triggers: [http://x, ftp://y]\n",
            "matches:\n  - trigger: hello\n    replace: hi\n  - trigger: bye\n    replace: bye\n",
            "vars:\n  - name: date\n    type: date\n    params:\n      format: '%Y-%m-%d'\n",
            "global_vars:\n  - name: clip\n    type: clipboard\n",
            "triggers:\n  - hello\n  - world\n",
            "word_separators: [\"'\", '.', '-']\n",
            "triggers: [->arrow, normal]\n",
        ];
        for input in valid {
            let strict: serde_norway::Value = serde_norway::from_str(input)
                .unwrap_or_else(|e| panic!("valid baseline should parse: {input:?}: {e}"));
            let lenient: serde_norway::Value = serde_norway::from_str(&lenient_flow_quote(input))
                .unwrap_or_else(|e| panic!("lenient should parse valid: {input:?}: {e}"));
            assert_eq!(strict, lenient, "value changed for valid: {input:?}");
            // parse_lenient must agree with the strict parse for valid input.
            let gated: serde_norway::Value = parse_lenient(input)
                .unwrap_or_else(|e| panic!("parse_lenient should succeed: {input:?}: {e}"));
            assert_eq!(strict, gated, "parse_lenient diverged for valid: {input:?}");
        }

        // (b) #2748-style YAML: strictly rejected, must be recovered with the right value.
        // (key, yaml, expected `triggers`/`word_separators` value)
        let fixed: &[(&str, &[&str])] = &[
            (
                "matches:\n  - triggers: [:->,:>-]\n    replace: smile\n",
                &[":->", ":>-"],
            ),
            (
                "matches:\n  - triggers: [:-), :-D, :>hello]\n    replace: smile\n",
                &[":-)", ":-D", ":>hello"],
            ),
            ("triggers: [:>x, :a, :b]\n", &[":>x", ":a", ":b"]),
            ("triggers: [>fold]\n", &[">fold"]),
            ("word_separators: [:sep]\n", &[":sep"]),
        ];
        for (input, expected) in fixed {
            let value: serde_norway::Value = parse_lenient(input)
                .unwrap_or_else(|e| panic!("#2748 input should be recovered: {input:?}: {e}"));
            // The sequence lives either at top level (`triggers:`/`word_separators:`) or
            // nested under `matches: - triggers:`.
            let list = value
                .get("triggers")
                .or_else(|| value.get("word_separators"))
                .or_else(|| {
                    value
                        .get("matches")
                        .and_then(|m| m.get(0))
                        .and_then(|m| m.get("triggers"))
                })
                .unwrap_or_else(|| panic!("expected a triggers key for {input:?}: {value:?}"))
                .as_sequence()
                .expect("expected a sequence");
            let got: Vec<&str> = list
                .iter()
                .map(|v| v.as_str().expect("expected strings"))
                .collect();
            assert_eq!(got, *expected, "recovered value mismatch for {input:?}");
        }
    }

    #[test]
    fn lenient_flow_quote_is_robust_against_pathological_input() {
        // None of these must panic or hang; for inputs that are not valid YAML the
        // helper just returns some string and parse_lenient returns the original error.
        let inputs = [
            "",
            "\n",
            "\r\n",
            "\u{0}",
            "[",
            "]",
            "{",
            "}",
            "][",
            "'",
            "\"",
            "'unterminated",
            "\"unterminated",
            "['",
            "[\"",
            "[:->",
            "[:",
            "[,]",
            "[:[:>x]]",
        ];
        for input in inputs {
            // Must terminate and must not panic.
            let _ = lenient_flow_quote(input);
            let _: Result<serde_norway::Value, _> = parse_lenient(input);
        }
        assert_eq!(lenient_flow_quote(""), "");

        // Deeply nested / long inputs must terminate (no exponential blow-up or hang).
        let big_open: String = "[".repeat(100_000);
        let _ = lenient_flow_quote(&big_open);
        let big_close: String = "]".repeat(100_000);
        let _ = lenient_flow_quote(&big_close);
        let big_alt: String = (0..50_000)
            .map(|i| if i % 2 == 0 { '[' } else { ']' })
            .collect();
        let _ = lenient_flow_quote(&big_alt);
    }
}
