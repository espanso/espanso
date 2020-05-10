/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use regex::{Captures, Regex};

lazy_static! {
    static ref ARG_REGEX: Regex = Regex::new("\\$(?P<pos>\\d+)\\$").unwrap();
}

pub fn render_args(text: &str, args: &Vec<String>) -> String {
    let result = ARG_REGEX.replace_all(text, |caps: &Captures| {
        let position_str = caps.name("pos").unwrap().as_str();
        let position = position_str.parse::<i32>().unwrap_or(-1);

        if position >= 0 && position < args.len() as i32 {
            args[position as usize].to_owned()
        } else {
            "".to_owned()
        }
    });

    result.to_string()
}

pub fn split_args(text: &str, delimiter: char, escape: char) -> Vec<String> {
    let mut output = vec![];

    // Make sure the text is not empty
    if text.is_empty() {
        return output;
    }

    let mut last = String::from("");
    let mut previous: char = char::from(0);
    text.chars().into_iter().for_each(|c| {
        if c == delimiter {
            if previous != escape {
                output.push(last.clone());
                last = String::from("");
            } else {
                last.push(c);
            }
        } else if c == escape {
            if previous == escape {
                last.push(c);
            }
        } else {
            last.push(c);
        }
        previous = c;
    });

    // Add the last one
    output.push(last);

    output
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_args_no_args() {
        let args = vec!["hello".to_owned()];
        assert_eq!(render_args("no args", &args), "no args")
    }

    #[test]
    fn test_render_args_one_arg() {
        let args = vec!["jon".to_owned()];
        assert_eq!(render_args("hello $0$", &args), "hello jon")
    }

    #[test]
    fn test_render_args_one_multiple_args() {
        let args = vec!["jon".to_owned(), "snow".to_owned()];
        assert_eq!(
            render_args("hello $0$, the $1$ is white", &args),
            "hello jon, the snow is white"
        )
    }

    #[test]
    fn test_render_args_out_of_range() {
        let args = vec!["jon".to_owned()];
        assert_eq!(render_args("hello $10$", &args), "hello ")
    }

    #[test]
    fn test_split_args_one_arg() {
        assert_eq!(split_args("jon", '/', '\\'), vec!["jon"])
    }

    #[test]
    fn test_split_args_two_args() {
        assert_eq!(split_args("jon/snow", '/', '\\'), vec!["jon", "snow"])
    }

    #[test]
    fn test_split_args_escaping() {
        assert_eq!(split_args("jon\\/snow", '/', '\\'), vec!["jon/snow"])
    }

    #[test]
    fn test_split_args_escaping_escape() {
        assert_eq!(split_args("jon\\\\snow", '/', '\\'), vec!["jon\\snow"])
    }

    #[test]
    fn test_split_args_empty() {
        let empty_vec: Vec<String> = vec![];
        assert_eq!(split_args("", '/', '\\'), empty_vec)
    }
}
