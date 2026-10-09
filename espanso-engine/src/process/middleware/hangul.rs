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

// When a Korean input method is active, the keys detected by espanso are the
// latin ones (for example "cap/"), but the input method composes them into
// Hangul syllables ("ㅊ메/"), so the number of characters on screen is lower
// than the trigger length. These utilities simulate the standard 2-set
// (Dubeolsik) composition to find out how many backspaces are needed to
// delete the typed trigger.

fn dubeolsik_jamo(key: char) -> Option<char> {
    let jamo = match key {
        'Q' => 'ㅃ',
        'W' => 'ㅉ',
        'E' => 'ㄸ',
        'R' => 'ㄲ',
        'T' => 'ㅆ',
        'O' => 'ㅒ',
        'P' => 'ㅖ',
        _ => match key.to_ascii_lowercase() {
            'q' => 'ㅂ',
            'w' => 'ㅈ',
            'e' => 'ㄷ',
            'r' => 'ㄱ',
            't' => 'ㅅ',
            'y' => 'ㅛ',
            'u' => 'ㅕ',
            'i' => 'ㅑ',
            'o' => 'ㅐ',
            'p' => 'ㅔ',
            'a' => 'ㅁ',
            's' => 'ㄴ',
            'd' => 'ㅇ',
            'f' => 'ㄹ',
            'g' => 'ㅎ',
            'h' => 'ㅗ',
            'j' => 'ㅓ',
            'k' => 'ㅏ',
            'l' => 'ㅣ',
            'z' => 'ㅋ',
            'x' => 'ㅌ',
            'c' => 'ㅊ',
            'v' => 'ㅍ',
            'b' => 'ㅠ',
            'n' => 'ㅜ',
            'm' => 'ㅡ',
            _ => return None,
        },
    };
    Some(jamo)
}

fn is_vowel(jamo: char) -> bool {
    ('ㅏ'..='ㅣ').contains(&jamo)
}

fn can_be_final(consonant: char) -> bool {
    !matches!(consonant, 'ㄸ' | 'ㅃ' | 'ㅉ')
}

fn combine_vowels(first: char, second: char) -> Option<char> {
    match (first, second) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

fn can_combine_finals(first: char, second: char) -> bool {
    matches!(
        (first, second),
        ('ㄱ' | 'ㅂ', 'ㅅ')
            | ('ㄴ', 'ㅈ' | 'ㅎ')
            | ('ㄹ', 'ㄱ' | 'ㅁ' | 'ㅂ' | 'ㅅ' | 'ㅌ' | 'ㅍ' | 'ㅎ')
    )
}

#[derive(Default)]
struct Syllable {
    initial: Option<char>,
    medial: Option<char>,
    finals: Vec<char>,
    // Number of keys that went into this syllable
    keys: usize,
}

impl Syllable {
    fn is_empty(&self) -> bool {
        self.keys == 0
    }
}

/// Return the number of backspaces needed to delete the given trigger keys
/// after they have been typed with the 2-set (Dubeolsik) Korean input method.
///
/// Each committed character takes one backspace, while the syllable that is
/// still being composed (when the trigger ends with a letter) is deleted one
/// key at a time.
pub fn dubeolsik_backspace_count(keys: &str) -> usize {
    let mut committed = 0;
    let mut current = Syllable::default();

    for key in keys.chars() {
        let Some(jamo) = dubeolsik_jamo(key) else {
            // Any other character commits the composition
            if !current.is_empty() {
                committed += 1;
            }
            current = Syllable::default();
            committed += 1;
            continue;
        };

        if is_vowel(jamo) {
            if let Some(moved) = current.finals.pop() {
                // The last final consonant moves to the next syllable
                committed += 1;
                current = Syllable {
                    initial: Some(moved),
                    medial: Some(jamo),
                    finals: Vec::new(),
                    keys: 2,
                };
                continue;
            }

            let medial = match current.medial {
                Some(medial) => combine_vowels(medial, jamo),
                None => Some(jamo),
            };
            if medial.is_some() {
                current.medial = medial;
                current.keys += 1;
                continue;
            }

            committed += 1;
            current = Syllable {
                medial: Some(jamo),
                keys: 1,
                ..Default::default()
            };
        } else {
            if current.initial.is_some() && current.medial.is_some() {
                let fits = match current.finals.as_slice() {
                    [] => can_be_final(jamo),
                    [first] => can_combine_finals(*first, jamo),
                    _ => false,
                };
                if fits {
                    current.finals.push(jamo);
                    current.keys += 1;
                    continue;
                }
            }

            if !current.is_empty() {
                committed += 1;
            }
            current = Syllable {
                initial: Some(jamo),
                keys: 1,
                ..Default::default()
            };
        }
    }

    committed + current.keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_syllables_take_one_backspace_each() {
        // ㅊ메/
        assert_eq!(dubeolsik_backspace_count("cap/"), 3);
        // 인/
        assert_eq!(dubeolsik_backspace_count("dls/"), 2);
        // 남/
        assert_eq!(dubeolsik_backspace_count("ska/"), 2);
    }

    #[test]
    fn non_letters_are_counted_as_is() {
        assert_eq!(dubeolsik_backspace_count("100/"), 4);
        assert_eq!(dubeolsik_backspace_count(":/"), 2);
    }

    #[test]
    fn final_consonant_moves_to_next_syllable() {
        // 하나/
        assert_eq!(dubeolsik_backspace_count("gksk/"), 3);
        // 닭이/ (ㄺ splits, ㄱ moves)
        assert_eq!(dubeolsik_backspace_count("ekfrdl/"), 3);
        assert_eq!(dubeolsik_backspace_count("ekfrl/"), 3);
    }

    #[test]
    fn compound_vowels_and_finals() {
        // 과/
        assert_eq!(dubeolsik_backspace_count("rhk/"), 2);
        // 값/
        assert_eq!(dubeolsik_backspace_count("rkqt/"), 2);
    }

    #[test]
    fn double_consonants_cannot_be_final() {
        // 가ㄸ/
        assert_eq!(dubeolsik_backspace_count("rkE/"), 3);
        // 갔/
        assert_eq!(dubeolsik_backspace_count("rkT/"), 2);
    }

    #[test]
    fn standalone_jamo() {
        // ㄱㄴ/
        assert_eq!(dubeolsik_backspace_count("rs/"), 3);
        // ㅏㄱ/
        assert_eq!(dubeolsik_backspace_count("kr/"), 3);
    }

    #[test]
    fn composing_syllable_is_deleted_key_by_key() {
        // "인" is still being composed: ㄴ, ㅣ and ㅇ are deleted one by one
        assert_eq!(dubeolsik_backspace_count("dls"), 3);
        // ";" is committed, then "과" is composed with 3 keys
        assert_eq!(dubeolsik_backspace_count(";rhk"), 4);
    }

    #[test]
    fn empty_trigger() {
        assert_eq!(dubeolsik_backspace_count(""), 0);
    }
}
