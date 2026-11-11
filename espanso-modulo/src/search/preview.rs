/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use unicode_segmentation::UnicodeSegmentation;

pub const MAX_PREVIEW_GRAPHEMES: usize = 80;

/// Flattens and bounds a preview without splitting a user-perceived character.
pub fn format_preview(preview: &str) -> String {
    let mut graphemes = preview.graphemes(true).map(|grapheme| {
        if grapheme.contains('\r') || grapheme.contains('\n') {
            " "
        } else if grapheme.contains('\0') {
            "�"
        } else {
            grapheme
        }
    });
    let mut formatted: String = graphemes
        .by_ref()
        .take(MAX_PREVIEW_GRAPHEMES + 1)
        .collect();

    if formatted.graphemes(true).count() > MAX_PREVIEW_GRAPHEMES {
        formatted = formatted
            .graphemes(true)
            .take(MAX_PREVIEW_GRAPHEMES - 3)
            .collect();
        formatted.push_str("...");
    }

    formatted
}

#[cfg(test)]
mod tests {
    use super::{format_preview, MAX_PREVIEW_GRAPHEMES};
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn truncation_preserves_grapheme_clusters() {
        let family_emoji = "👨‍👩‍👧‍👦";
        let combining_character = "e\u{301}";
        let source = format!("{}{}tail", combining_character.repeat(76), family_emoji);

        let preview = format_preview(&source);

        assert_eq!(
            preview,
            format!("{}{}...", combining_character.repeat(76), family_emoji)
        );
        assert_eq!(preview.graphemes(true).count(), MAX_PREVIEW_GRAPHEMES);
    }

    #[test]
    fn newlines_and_nul_are_safe_for_the_native_ui() {
        assert_eq!(format_preview("first\r\nsecond\0"), "first second�");
    }
}
