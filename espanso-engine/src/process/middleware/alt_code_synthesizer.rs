/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2022 Federico Terzi
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

use std::cell::RefCell;

use log::debug;

use super::super::Middleware;
use crate::event::{
    effect::TextInjectRequest,
    input::{Key, Status},
    Event, EventType,
};

pub trait AltCodeSynthEnabledProvider {
    fn is_alt_code_synthesizer_enabled(&self) -> bool;
}

pub struct AltCodeSynthesizerMiddleware<'a> {
    code_buffer: RefCell<Option<String>>,
    enabled_state_provider: &'a dyn AltCodeSynthEnabledProvider,
}

impl<'a> AltCodeSynthesizerMiddleware<'a> {
    pub fn new(enabled_state_provider: &'a dyn AltCodeSynthEnabledProvider) -> Self {
        Self {
            code_buffer: RefCell::new(None),
            enabled_state_provider,
        }
    }
}

impl<'a> Middleware for AltCodeSynthesizerMiddleware<'a> {
    fn name(&self) -> &'static str {
        "alt_code_synthesizer"
    }

    fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
        match &event.etype {
            EventType::Keyboard(keyboard_event)
                if self
                    .enabled_state_provider
                    .is_alt_code_synthesizer_enabled() =>
            {
                let mut code_buffer = self.code_buffer.borrow_mut();

                if keyboard_event.status == Status::Pressed {
                    if let Key::Alt = &keyboard_event.key {
                        *code_buffer = Some(String::new());
                    } else if let Some(buffer) = &mut *code_buffer {
                        match keyboard_event.key {
                            Key::Numpad0 => buffer.push('0'),
                            Key::Numpad1 => buffer.push('1'),
                            Key::Numpad2 => buffer.push('2'),
                            Key::Numpad3 => buffer.push('3'),
                            Key::Numpad4 => buffer.push('4'),
                            Key::Numpad5 => buffer.push('5'),
                            Key::Numpad6 => buffer.push('6'),
                            Key::Numpad7 => buffer.push('7'),
                            Key::Numpad8 => buffer.push('8'),
                            Key::Numpad9 => buffer.push('9'),
                            _ => {}
                        }
                    }
                } else if keyboard_event.key == Key::Alt {
                    if let Some(codes) = &*code_buffer {
                        if let Some(target_char) = convert_buffer_into_char(codes) {
                            dispatch(Event::caused_by(
                                event.source_id,
                                EventType::TextInject(TextInjectRequest {
                                    text: target_char,
                                    ..Default::default()
                                }),
                            ));
                        }
                    }

                    *code_buffer = None;
                }
            }
            _ => {}
        }

        event
    }
}

fn convert_buffer_into_char(buffer: &str) -> Option<String> {
    if buffer.is_empty() {
        debug!("unable to generate ALT code as the buffer is empty");
        return None;
    }

    if !buffer.chars().all(char::is_numeric) {
        debug!("unable to generate ALT code as some of the buffer chars are not numeric");
        return None;
    }

    // According to: https://en.wikipedia.org/wiki/Alt_code
    // The conversion works as follow:
    // * If the number is smaller than 256 and does not start with 0, then we use a patched version of the CP437 encoding
    // * If the number is smaller than 256 and starts with 0 we use the CP1252 encoding
    // * If the number is greater or equal than 256 we use the unicode encoding

    let code = buffer.parse::<u32>().ok()?;
    let unicode_code = if code >= 256 {
        // Unicode
        code
    } else if buffer.starts_with('0') {
        // CP1252
        convert_cp1252_code_to_unicode(code)?
    } else {
        // CP437
        convert_cp437_code_to_unicode(code)?
    };

    char::from_u32(unicode_code).map(|c| c.to_string())
}

// Taken from: https://altcodeunicode.com/
fn convert_cp437_code_to_unicode(code: u32) -> Option<u32> {
    match code {
        0 => Some(0x0000),   // Control character - null (NUL)
        1 => Some(0x263A),   // White smiling face, smiley face
        2 => Some(0x263B),   // Black smiling face
        3 => Some(0x2665),   // Black heart suit
        4 => Some(0x2666),   // Black diamond suit
        5 => Some(0x2663),   // Black club suit
        6 => Some(0x2660),   // Black spade suit
        7 => Some(0x2022),   // Bullet
        8 => Some(0x25D8),   // Inverse bullet
        9 => Some(0x25CB),   // White circle
        10 => Some(0x25D9),  // Inverse white circle
        11 => Some(0x2642),  // Male sign, mars, alchemical symbol for iron
        12 => Some(0x2640),  // Female sign, venus, alchemical symbol for copper
        13 => Some(0x266A),  // Eighth note, quaver
        14 => Some(0x266B),  // Beamed eighth notes, barred eighth notes, beamed quavers
        15 => Some(0x263C),  // White sun with rays
        16 => Some(0x25BA),  // Black right-pointing pointer
        17 => Some(0x25C4),  // Black left-pointing pointer
        18 => Some(0x2195),  // Up down arrow
        19 => Some(0x203C),  // Double exclamation mark
        20 => Some(0x00B6),  // Pilcrow sign, paragraph sign
        21 => Some(0x00A7),  // Section sign
        22 => Some(0x25AC),  // Black rectangle
        23 => Some(0x21A8),  // Up down arrow with base
        24 => Some(0x2191),  // Upwards arrow
        25 => Some(0x2193),  // Downwards arrow
        26 => Some(0x2192),  // Rightwards arrow, Z notation total function
        27 => Some(0x2190),  // Leftwards arrow
        28 => Some(0x221F),  // Right angle
        29 => Some(0x2194),  // Left right arrow, Z notation relation
        30 => Some(0x25B2),  // Black up-pointing triangle
        31 => Some(0x25BC),  // Black down-pointing triangle
        32 => Some(0x0020),  // Space
        33 => Some(0x0021),  // Exclamation mark, factorial
        34 => Some(0x0022),  // Quotation mark
        35 => Some(0x0023),  // Number sign, pound sign, hash, crosshatch, octothorpe
        36 => Some(0x0024),  // Dollar sign, milréis, escudo
        37 => Some(0x0025),  // Percent sign
        38 => Some(0x0026),  // Ampersand
        39 => Some(0x0027),  // Apostrophe
        40 => Some(0x0028),  // Left parenthesis, opening parenthesis
        41 => Some(0x0029),  // Right parenthesis, closing parenthesis
        42 => Some(0x002A),  // Asterisk, star
        43 => Some(0x002B),  // Plus sign
        44 => Some(0x002C),  // Comma, decimal separator
        45 => Some(0x002D),  // Hyphen, minus sign
        46 => Some(0x002E),  // Full stop, period, dot, decimal point
        47 => Some(0x002F),  // Solidus, slash, forward slash, virgule
        48 => Some(0x0030),  // Digit zero
        49 => Some(0x0031),  // Digit one
        50 => Some(0x0032),  // Digit two
        51 => Some(0x0033),  // Digit three
        52 => Some(0x0034),  // Digit four
        53 => Some(0x0035),  // Digit five
        54 => Some(0x0036),  // Digit six
        55 => Some(0x0037),  // Digit seven
        56 => Some(0x0038),  // Digit eight
        57 => Some(0x0039),  // Digit nine
        58 => Some(0x003A),  // Colon
        59 => Some(0x003B),  // Semicolon
        60 => Some(0x003C),  // Less-than sign
        61 => Some(0x003D),  // Equals sign
        62 => Some(0x003E),  // Greater-than sign
        63 => Some(0x003F),  // Question mark
        64 => Some(0x0040),  // Commercial at, at sign
        65 => Some(0x0041),  // Latin capital letter A
        66 => Some(0x0042),  // Latin capital letter B
        67 => Some(0x0043),  // Latin capital letter C
        68 => Some(0x0044),  // Latin capital letter D
        69 => Some(0x0045),  // Latin capital letter E
        70 => Some(0x0046),  // Latin capital letter F
        71 => Some(0x0047),  // Latin capital letter G
        72 => Some(0x0048),  // Latin capital letter H
        73 => Some(0x0049),  // Latin capital letter I
        74 => Some(0x004A),  // Latin capital letter J
        75 => Some(0x004B),  // Latin capital letter K
        76 => Some(0x004C),  // Latin capital letter L
        77 => Some(0x004D),  // Latin capital letter M
        78 => Some(0x004E),  // Latin capital letter N
        79 => Some(0x004F),  // Latin capital letter O
        80 => Some(0x0050),  // Latin capital letter P
        81 => Some(0x0051),  // Latin capital letter Q
        82 => Some(0x0052),  // Latin capital letter R
        83 => Some(0x0053),  // Latin capital letter S
        84 => Some(0x0054),  // Latin capital letter T
        85 => Some(0x0055),  // Latin capital letter U
        86 => Some(0x0056),  // Latin capital letter V
        87 => Some(0x0057),  // Latin capital letter W
        88 => Some(0x0058),  // Latin capital letter X
        89 => Some(0x0059),  // Latin capital letter Y
        90 => Some(0x005A),  // Latin capital letter Z
        91 => Some(0x005B),  // Left square bracket, opening square bracket
        92 => Some(0x005C),  // Reverse solidus, back slash
        93 => Some(0x005D),  // Right square bracket, closing square bracket
        94 => Some(0x005E),  // Circumflex accent
        95 => Some(0x005F),  // Low line, underscore
        96 => Some(0x0060),  // Grave accent
        97 => Some(0x0061),  // Latin small letter a
        98 => Some(0x0062),  // Latin small letter b
        99 => Some(0x0063),  // Latin small letter c
        100 => Some(0x0064), // Latin small letter d
        101 => Some(0x0065), // Latin small letter e
        102 => Some(0x0066), // Latin small letter f
        103 => Some(0x0067), // Latin small letter g
        104 => Some(0x0068), // Latin small letter h
        105 => Some(0x0069), // Latin small letter i
        106 => Some(0x006A), // Latin small letter j
        107 => Some(0x006B), // Latin small letter k
        108 => Some(0x006C), // Latin small letter l
        109 => Some(0x006D), // Latin small letter m
        110 => Some(0x006E), // Latin small letter n
        111 => Some(0x006F), // Latin small letter o
        112 => Some(0x0070), // Latin small letter p
        113 => Some(0x0071), // Latin small letter q
        114 => Some(0x0072), // Latin small letter r
        115 => Some(0x0073), // Latin small letter s
        116 => Some(0x0074), // Latin small letter t
        117 => Some(0x0075), // Latin small letter u
        118 => Some(0x0076), // Latin small letter v
        119 => Some(0x0077), // Latin small letter w
        120 => Some(0x0078), // Latin small letter x
        121 => Some(0x0079), // Latin small letter y
        122 => Some(0x007A), // Latin small letter z
        123 => Some(0x007B), // Left curly bracket, opening curly bracket, left brace
        124 => Some(0x007C), // Vertical line, vertical bar
        125 => Some(0x007D), // Right curly bracket, closing curly bracket, right brace
        126 => Some(0x007E), // Tilde
        127 => Some(0x2302), // House
        128 => Some(0x00C7), // Latin capital letter C with cedilla
        129 => Some(0x00FC), // Latin small letter u with diaeresis
        130 => Some(0x00E9), // Latin small letter e with acute
        131 => Some(0x00E2), // Latin small letter a with circumflex
        132 => Some(0x00E4), // Latin small letter a with diaeresis
        133 => Some(0x00E0), // Latin small letter a with grave
        134 => Some(0x00E5), // Latin small letter a with ring above
        135 => Some(0x00E7), // Latin small letter c with cedilla
        136 => Some(0x00EA), // Latin small letter e with circumflex
        137 => Some(0x00EB), // Latin small letter e with diaeresis
        138 => Some(0x00E8), // Latin small letter e with grave
        139 => Some(0x00EF), // Latin small letter i with diaeresis
        140 => Some(0x00EE), // Latin small letter i with circumflex
        141 => Some(0x00EC), // Latin small letter i with grave
        142 => Some(0x00C4), // Latin capital letter A with diaeresis
        143 => Some(0x00C5), // Latin capital letter A with ring above
        144 => Some(0x00C9), // Latin capital letter E with acute
        145 => Some(0x00E6), // Latin small letter ae, ash (from Old English æsc)
        146 => Some(0x00C6), // Latin capital letter AE
        147 => Some(0x00F4), // Latin small letter o with circumflex
        148 => Some(0x00F6), // Latin small letter o with diaeresis
        149 => Some(0x00F2), // Latin small letter o with grave
        150 => Some(0x00FB), // Latin small letter u with circumflex
        151 => Some(0x00F9), // Latin small letter u with grave
        152 => Some(0x00FF), // Latin small letter y with diaeresis
        153 => Some(0x00D6), // Latin capital letter O with diaeresis
        154 => Some(0x00DC), // Latin capital letter U with diaeresis
        155 => Some(0x00A2), // Cent sign
        156 => Some(0x00A3), // Pound sign, pound sterling, Irish punt, lira sign
        157 => Some(0x00A5), // Yen sign, yuan sign
        158 => Some(0x20A7), // Peseta sign
        159 => Some(0x0192), // Latin small letter f with hook, florin currency symbol, function symbol
        160 => Some(0x00E1), // Latin small letter a with acute
        161 => Some(0x00ED), // Latin small letter i with acute
        162 => Some(0x00F3), // Latin small letter o with acute
        163 => Some(0x00FA), // Latin small letter u with acute
        164 => Some(0x00F1), // Latin small letter n with tilde, small letter enye
        165 => Some(0x00D1), // Latin capital letter N with tilde, capital letter enye
        166 => Some(0x00AA), // Feminine ordinal indicator
        167 => Some(0x00BA), // Masculine ordinal indicator
        168 => Some(0x00BF), // Inverted question mark, turned question mark
        169 => Some(0x2310), // Reversed not sign, beginning of line
        170 => Some(0x00AC), // Not sign, angled dash
        171 => Some(0x00BD), // Vulgar fraction one half
        172 => Some(0x00BC), // Vulgar fraction one quarter
        173 => Some(0x00A1), // Inverted exclamation mark
        174 => Some(0x00AB), // Left-pointing double angle quotation mark, left guillemet, chevrons (in typography)
        175 => Some(0x00BB), // Right-pointing double angle quotation mark, right guillemet
        176 => Some(0x2591), // Light shade
        177 => Some(0x2592), // Medium shade, speckles fill, dotted fill
        178 => Some(0x2593), // Dark shade
        179 => Some(0x2502), // Box drawings light vertical
        180 => Some(0x2524), // Box drawings light vertical and left
        181 => Some(0x2561), // Box drawings vertical single and left double
        182 => Some(0x2562), // Box drawings vertical double and left single
        183 => Some(0x2556), // Box drawings down double and left single
        184 => Some(0x2555), // Box drawings down single and left double
        185 => Some(0x2563), // Box drawings double vertical and left
        186 => Some(0x2551), // Box drawings double vertical
        187 => Some(0x2557), // Box drawings double down and left
        188 => Some(0x255D), // Box drawings double up and left
        189 => Some(0x255C), // Box drawings up double and left single
        190 => Some(0x255B), // Box drawings up single and left double
        191 => Some(0x2510), // Box drawings light down and left
        192 => Some(0x2514), // Box drawings light up and right
        193 => Some(0x2534), // Box drawings light up and horizontal
        194 => Some(0x252C), // Box drawings light down and horizontal
        195 => Some(0x251C), // Box drawings light vertical and right
        196 => Some(0x2500), // Box drawings light horizontal
        197 => Some(0x253C), // Box drawings light vertical and horizontal
        198 => Some(0x255E), // Box drawings vertical single and right double
        199 => Some(0x255F), // Box drawings vertical double and right single
        200 => Some(0x255A), // Box drawings double up and right
        201 => Some(0x2554), // Box drawings double down and right
        202 => Some(0x2569), // Box drawings double up and horizontal
        203 => Some(0x2566), // Box drawings double down and horizontal
        204 => Some(0x2560), // Box drawings double vertical and right
        205 => Some(0x2550), // Box drawings double horizontal
        206 => Some(0x256C), // Box drawings double vertical and horizontal
        207 => Some(0x2567), // Box drawings up single and horizontal double
        208 => Some(0x2568), // Box drawings up double and horizontal single
        209 => Some(0x2564), // Box drawings down single and horizontal double
        210 => Some(0x2565), // Box drawings down double and horizontal single
        211 => Some(0x2559), // Box drawings up double and right single
        212 => Some(0x2558), // Box drawings up single and right double
        213 => Some(0x2552), // Box drawings down single and right double
        214 => Some(0x2553), // Box drawings down double and right single
        215 => Some(0x256B), // Box drawings vertical double and horizontal single
        216 => Some(0x256A), // Box drawings vertical single and horizontal double
        217 => Some(0x2518), // Box drawings light up and left
        218 => Some(0x250C), // Box drawings light down and right
        219 => Some(0x2588), // Full block, solid block
        220 => Some(0x2584), // Lower half block
        221 => Some(0x258C), // Left half block
        222 => Some(0x2590), // Right half block
        223 => Some(0x2580), // Upper half block
        224 => Some(0x03B1), // Greek small letter alpha
        225 => Some(0x00DF), // Latin small letter sharp s, eszett
        226 => Some(0x0393), // Greek capital letter gamma
        227 => Some(0x03C0), // Greek small letter pi
        228 => Some(0x03A3), // Greek capital letter sigma
        229 => Some(0x03C3), // Greek small letter sigma
        230 => Some(0x00B5), // Micro sign
        231 => Some(0x03A4), // Greek capital letter tau
        232 => Some(0x03A6), // Greek capital letter phi
        233 => Some(0x0398), // Greek capital letter theta
        234 => Some(0x03A9), // Greek capital letter omega
        235 => Some(0x03B4), // Greek small letter delta
        236 => Some(0x221E), // Infinity
        237 => Some(0x03C6), // Greek small letter phi
        238 => Some(0x03B5), // Greek small letter epsilon
        239 => Some(0x2229), // Intersection
        240 => Some(0x2261), // Identical to
        241 => Some(0x00B1), // Plus-minus sign
        242 => Some(0x2265), // Greater-than or equal to
        243 => Some(0x2264), // Less-than or equal to
        244 => Some(0x2320), // Top half integral
        245 => Some(0x2321), // Bottom half integral
        246 => Some(0x00F7), // Division sign, obelus
        247 => Some(0x2248), // Almost equal to, asymptotic to
        248 => Some(0x00B0), // Degree sign
        249 => Some(0x2219), // Bullet operator
        250 => Some(0x00B7), // Middle dot, midpoint (in typography), interpunct, Georgian comma, Greek ano teleia
        251 => Some(0x221A), // Square root, radical sign
        252 => Some(0x207F), // Superscript Latin small letter n
        253 => Some(0x00B2), // Superscript two, squared
        254 => Some(0x25A0), // Black square
        255 => Some(0x00A0), // No-break space, non-breaking space, nbsp
        _ => None,
    }
}

// Taken from here: https://unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WINDOWS/CP1252.TXT
/*
#
#    Name:     cp1252 to Unicode table
#    Unicode version: 2.0
#    Table version: 2.01
#    Table format:  Format A
#    Date:          04/15/98
#
#    Contact:       Shawn.Steele@microsoft.com
#
#    General notes: none
#
#    Format: Three tab-separated columns
#        Column #1 is the cp1252 code (in hex)
#        Column #2 is the Unicode (in hex as 0xXXXX)
#        Column #3 is the Unicode name (follows a comment sign, '#')
#
#    The entries are in cp1252 order
#
*/
fn convert_cp1252_code_to_unicode(code: u32) -> Option<u32> {
    match code {
        0x00 => Some(0x0000), // #NULL
        0x01 => Some(0x0001), // #START OF HEADING
        0x02 => Some(0x0002), // #START OF TEXT
        0x03 => Some(0x0003), // #END OF TEXT
        0x04 => Some(0x0004), // #END OF TRANSMISSION
        0x05 => Some(0x0005), // #ENQUIRY
        0x06 => Some(0x0006), // #ACKNOWLEDGE
        0x07 => Some(0x0007), // #BELL
        0x08 => Some(0x0008), // #BACKSPACE
        0x09 => Some(0x0009), // #HORIZONTAL TABULATION
        0x0A => Some(0x000A), // #LINE FEED
        0x0B => Some(0x000B), // #VERTICAL TABULATION
        0x0C => Some(0x000C), // #FORM FEED
        0x0D => Some(0x000D), // #CARRIAGE RETURN
        0x0E => Some(0x000E), // #SHIFT OUT
        0x0F => Some(0x000F), // #SHIFT IN
        0x10 => Some(0x0010), // #DATA LINK ESCAPE
        0x11 => Some(0x0011), // #DEVICE CONTROL ONE
        0x12 => Some(0x0012), // #DEVICE CONTROL TWO
        0x13 => Some(0x0013), // #DEVICE CONTROL THREE
        0x14 => Some(0x0014), // #DEVICE CONTROL FOUR
        0x15 => Some(0x0015), // #NEGATIVE ACKNOWLEDGE
        0x16 => Some(0x0016), // #SYNCHRONOUS IDLE
        0x17 => Some(0x0017), // #END OF TRANSMISSION BLOCK
        0x18 => Some(0x0018), // #CANCEL
        0x19 => Some(0x0019), // #END OF MEDIUM
        0x1A => Some(0x001A), // #SUBSTITUTE
        0x1B => Some(0x001B), // #ESCAPE
        0x1C => Some(0x001C), // #FILE SEPARATOR
        0x1D => Some(0x001D), // #GROUP SEPARATOR
        0x1E => Some(0x001E), // #RECORD SEPARATOR
        0x1F => Some(0x001F), // #UNIT SEPARATOR
        0x20 => Some(0x0020), // #SPACE
        0x21 => Some(0x0021), // #EXCLAMATION MARK
        0x22 => Some(0x0022), // #QUOTATION MARK
        0x23 => Some(0x0023), // #NUMBER SIGN
        0x24 => Some(0x0024), // #DOLLAR SIGN
        0x25 => Some(0x0025), // #PERCENT SIGN
        0x26 => Some(0x0026), // #AMPERSAND
        0x27 => Some(0x0027), // #APOSTROPHE
        0x28 => Some(0x0028), // #LEFT PARENTHESIS
        0x29 => Some(0x0029), // #RIGHT PARENTHESIS
        0x2A => Some(0x002A), // #ASTERISK
        0x2B => Some(0x002B), // #PLUS SIGN
        0x2C => Some(0x002C), // #COMMA
        0x2D => Some(0x002D), // #HYPHEN-MINUS
        0x2E => Some(0x002E), // #FULL STOP
        0x2F => Some(0x002F), // #SOLIDUS
        0x30 => Some(0x0030), // #DIGIT ZERO
        0x31 => Some(0x0031), // #DIGIT ONE
        0x32 => Some(0x0032), // #DIGIT TWO
        0x33 => Some(0x0033), // #DIGIT THREE
        0x34 => Some(0x0034), // #DIGIT FOUR
        0x35 => Some(0x0035), // #DIGIT FIVE
        0x36 => Some(0x0036), // #DIGIT SIX
        0x37 => Some(0x0037), // #DIGIT SEVEN
        0x38 => Some(0x0038), // #DIGIT EIGHT
        0x39 => Some(0x0039), // #DIGIT NINE
        0x3A => Some(0x003A), // #COLON
        0x3B => Some(0x003B), // #SEMICOLON
        0x3C => Some(0x003C), // #LESS-THAN SIGN
        0x3D => Some(0x003D), // #EQUALS SIGN
        0x3E => Some(0x003E), // #GREATER-THAN SIGN
        0x3F => Some(0x003F), // #QUESTION MARK
        0x40 => Some(0x0040), // #COMMERCIAL AT
        0x41 => Some(0x0041), // #LATIN CAPITAL LETTER A
        0x42 => Some(0x0042), // #LATIN CAPITAL LETTER B
        0x43 => Some(0x0043), // #LATIN CAPITAL LETTER C
        0x44 => Some(0x0044), // #LATIN CAPITAL LETTER D
        0x45 => Some(0x0045), // #LATIN CAPITAL LETTER E
        0x46 => Some(0x0046), // #LATIN CAPITAL LETTER F
        0x47 => Some(0x0047), // #LATIN CAPITAL LETTER G
        0x48 => Some(0x0048), // #LATIN CAPITAL LETTER H
        0x49 => Some(0x0049), // #LATIN CAPITAL LETTER I
        0x4A => Some(0x004A), // #LATIN CAPITAL LETTER J
        0x4B => Some(0x004B), // #LATIN CAPITAL LETTER K
        0x4C => Some(0x004C), // #LATIN CAPITAL LETTER L
        0x4D => Some(0x004D), // #LATIN CAPITAL LETTER M
        0x4E => Some(0x004E), // #LATIN CAPITAL LETTER N
        0x4F => Some(0x004F), // #LATIN CAPITAL LETTER O
        0x50 => Some(0x0050), // #LATIN CAPITAL LETTER P
        0x51 => Some(0x0051), // #LATIN CAPITAL LETTER Q
        0x52 => Some(0x0052), // #LATIN CAPITAL LETTER R
        0x53 => Some(0x0053), // #LATIN CAPITAL LETTER S
        0x54 => Some(0x0054), // #LATIN CAPITAL LETTER T
        0x55 => Some(0x0055), // #LATIN CAPITAL LETTER U
        0x56 => Some(0x0056), // #LATIN CAPITAL LETTER V
        0x57 => Some(0x0057), // #LATIN CAPITAL LETTER W
        0x58 => Some(0x0058), // #LATIN CAPITAL LETTER X
        0x59 => Some(0x0059), // #LATIN CAPITAL LETTER Y
        0x5A => Some(0x005A), // #LATIN CAPITAL LETTER Z
        0x5B => Some(0x005B), // #LEFT SQUARE BRACKET
        0x5C => Some(0x005C), // #REVERSE SOLIDUS
        0x5D => Some(0x005D), // #RIGHT SQUARE BRACKET
        0x5E => Some(0x005E), // #CIRCUMFLEX ACCENT
        0x5F => Some(0x005F), // #LOW LINE
        0x60 => Some(0x0060), // #GRAVE ACCENT
        0x61 => Some(0x0061), // #LATIN SMALL LETTER A
        0x62 => Some(0x0062), // #LATIN SMALL LETTER B
        0x63 => Some(0x0063), // #LATIN SMALL LETTER C
        0x64 => Some(0x0064), // #LATIN SMALL LETTER D
        0x65 => Some(0x0065), // #LATIN SMALL LETTER E
        0x66 => Some(0x0066), // #LATIN SMALL LETTER F
        0x67 => Some(0x0067), // #LATIN SMALL LETTER G
        0x68 => Some(0x0068), // #LATIN SMALL LETTER H
        0x69 => Some(0x0069), // #LATIN SMALL LETTER I
        0x6A => Some(0x006A), // #LATIN SMALL LETTER J
        0x6B => Some(0x006B), // #LATIN SMALL LETTER K
        0x6C => Some(0x006C), // #LATIN SMALL LETTER L
        0x6D => Some(0x006D), // #LATIN SMALL LETTER M
        0x6E => Some(0x006E), // #LATIN SMALL LETTER N
        0x6F => Some(0x006F), // #LATIN SMALL LETTER O
        0x70 => Some(0x0070), // #LATIN SMALL LETTER P
        0x71 => Some(0x0071), // #LATIN SMALL LETTER Q
        0x72 => Some(0x0072), // #LATIN SMALL LETTER R
        0x73 => Some(0x0073), // #LATIN SMALL LETTER S
        0x74 => Some(0x0074), // #LATIN SMALL LETTER T
        0x75 => Some(0x0075), // #LATIN SMALL LETTER U
        0x76 => Some(0x0076), // #LATIN SMALL LETTER V
        0x77 => Some(0x0077), // #LATIN SMALL LETTER W
        0x78 => Some(0x0078), // #LATIN SMALL LETTER X
        0x79 => Some(0x0079), // #LATIN SMALL LETTER Y
        0x7A => Some(0x007A), // #LATIN SMALL LETTER Z
        0x7B => Some(0x007B), // #LEFT CURLY BRACKET
        0x7C => Some(0x007C), // #VERTICAL LINE
        0x7D => Some(0x007D), // #RIGHT CURLY BRACKET
        0x7E => Some(0x007E), // #TILDE
        0x7F => Some(0x007F), // #DELETE
        0x80 => Some(0x20AC), // #EURO SIGN
        0x82 => Some(0x201A), // #SINGLE LOW-9 QUOTATION MARK
        0x83 => Some(0x0192), // #LATIN SMALL LETTER F WITH HOOK
        0x84 => Some(0x201E), // #DOUBLE LOW-9 QUOTATION MARK
        0x85 => Some(0x2026), // #HORIZONTAL ELLIPSIS
        0x86 => Some(0x2020), // #DAGGER
        0x87 => Some(0x2021), // #DOUBLE DAGGER
        0x88 => Some(0x02C6), // #MODIFIER LETTER CIRCUMFLEX ACCENT
        0x89 => Some(0x2030), // #PER MILLE SIGN
        0x8A => Some(0x0160), // #LATIN CAPITAL LETTER S WITH CARON
        0x8B => Some(0x2039), // #SINGLE LEFT-POINTING ANGLE QUOTATION MARK
        0x8C => Some(0x0152), // #LATIN CAPITAL LIGATURE OE
        0x8E => Some(0x017D), // #LATIN CAPITAL LETTER Z WITH CARON
        0x91 => Some(0x2018), // #LEFT SINGLE QUOTATION MARK
        0x92 => Some(0x2019), // #RIGHT SINGLE QUOTATION MARK
        0x93 => Some(0x201C), // #LEFT DOUBLE QUOTATION MARK
        0x94 => Some(0x201D), // #RIGHT DOUBLE QUOTATION MARK
        0x95 => Some(0x2022), // #BULLET
        0x96 => Some(0x2013), // #EN DASH
        0x97 => Some(0x2014), // #EM DASH
        0x98 => Some(0x02DC), // #SMALL TILDE
        0x99 => Some(0x2122), // #TRADE MARK SIGN
        0x9A => Some(0x0161), // #LATIN SMALL LETTER S WITH CARON
        0x9B => Some(0x203A), // #SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
        0x9C => Some(0x0153), // #LATIN SMALL LIGATURE OE
        0x9E => Some(0x017E), // #LATIN SMALL LETTER Z WITH CARON
        0x9F => Some(0x0178), // #LATIN CAPITAL LETTER Y WITH DIAERESIS
        0xA0 => Some(0x00A0), // #NO-BREAK SPACE
        0xA1 => Some(0x00A1), // #INVERTED EXCLAMATION MARK
        0xA2 => Some(0x00A2), // #CENT SIGN
        0xA3 => Some(0x00A3), // #POUND SIGN
        0xA4 => Some(0x00A4), // #CURRENCY SIGN
        0xA5 => Some(0x00A5), // #YEN SIGN
        0xA6 => Some(0x00A6), // #BROKEN BAR
        0xA7 => Some(0x00A7), // #SECTION SIGN
        0xA8 => Some(0x00A8), // #DIAERESIS
        0xA9 => Some(0x00A9), // #COPYRIGHT SIGN
        0xAA => Some(0x00AA), // #FEMININE ORDINAL INDICATOR
        0xAB => Some(0x00AB), // #LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
        0xAC => Some(0x00AC), // #NOT SIGN
        0xAD => Some(0x00AD), // #SOFT HYPHEN
        0xAE => Some(0x00AE), // #REGISTERED SIGN
        0xAF => Some(0x00AF), // #MACRON
        0xB0 => Some(0x00B0), // #DEGREE SIGN
        0xB1 => Some(0x00B1), // #PLUS-MINUS SIGN
        0xB2 => Some(0x00B2), // #SUPERSCRIPT TWO
        0xB3 => Some(0x00B3), // #SUPERSCRIPT THREE
        0xB4 => Some(0x00B4), // #ACUTE ACCENT
        0xB5 => Some(0x00B5), // #MICRO SIGN
        0xB6 => Some(0x00B6), // #PILCROW SIGN
        0xB7 => Some(0x00B7), // #MIDDLE DOT
        0xB8 => Some(0x00B8), // #CEDILLA
        0xB9 => Some(0x00B9), // #SUPERSCRIPT ONE
        0xBA => Some(0x00BA), // #MASCULINE ORDINAL INDICATOR
        0xBB => Some(0x00BB), // #RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
        0xBC => Some(0x00BC), // #VULGAR FRACTION ONE QUARTER
        0xBD => Some(0x00BD), // #VULGAR FRACTION ONE HALF
        0xBE => Some(0x00BE), // #VULGAR FRACTION THREE QUARTERS
        0xBF => Some(0x00BF), // #INVERTED QUESTION MARK
        0xC0 => Some(0x00C0), // #LATIN CAPITAL LETTER A WITH GRAVE
        0xC1 => Some(0x00C1), // #LATIN CAPITAL LETTER A WITH ACUTE
        0xC2 => Some(0x00C2), // #LATIN CAPITAL LETTER A WITH CIRCUMFLEX
        0xC3 => Some(0x00C3), // #LATIN CAPITAL LETTER A WITH TILDE
        0xC4 => Some(0x00C4), // #LATIN CAPITAL LETTER A WITH DIAERESIS
        0xC5 => Some(0x00C5), // #LATIN CAPITAL LETTER A WITH RING ABOVE
        0xC6 => Some(0x00C6), // #LATIN CAPITAL LETTER AE
        0xC7 => Some(0x00C7), // #LATIN CAPITAL LETTER C WITH CEDILLA
        0xC8 => Some(0x00C8), // #LATIN CAPITAL LETTER E WITH GRAVE
        0xC9 => Some(0x00C9), // #LATIN CAPITAL LETTER E WITH ACUTE
        0xCA => Some(0x00CA), // #LATIN CAPITAL LETTER E WITH CIRCUMFLEX
        0xCB => Some(0x00CB), // #LATIN CAPITAL LETTER E WITH DIAERESIS
        0xCC => Some(0x00CC), // #LATIN CAPITAL LETTER I WITH GRAVE
        0xCD => Some(0x00CD), // #LATIN CAPITAL LETTER I WITH ACUTE
        0xCE => Some(0x00CE), // #LATIN CAPITAL LETTER I WITH CIRCUMFLEX
        0xCF => Some(0x00CF), // #LATIN CAPITAL LETTER I WITH DIAERESIS
        0xD0 => Some(0x00D0), // #LATIN CAPITAL LETTER ETH
        0xD1 => Some(0x00D1), // #LATIN CAPITAL LETTER N WITH TILDE
        0xD2 => Some(0x00D2), // #LATIN CAPITAL LETTER O WITH GRAVE
        0xD3 => Some(0x00D3), // #LATIN CAPITAL LETTER O WITH ACUTE
        0xD4 => Some(0x00D4), // #LATIN CAPITAL LETTER O WITH CIRCUMFLEX
        0xD5 => Some(0x00D5), // #LATIN CAPITAL LETTER O WITH TILDE
        0xD6 => Some(0x00D6), // #LATIN CAPITAL LETTER O WITH DIAERESIS
        0xD7 => Some(0x00D7), // #MULTIPLICATION SIGN
        0xD8 => Some(0x00D8), // #LATIN CAPITAL LETTER O WITH STROKE
        0xD9 => Some(0x00D9), // #LATIN CAPITAL LETTER U WITH GRAVE
        0xDA => Some(0x00DA), // #LATIN CAPITAL LETTER U WITH ACUTE
        0xDB => Some(0x00DB), // #LATIN CAPITAL LETTER U WITH CIRCUMFLEX
        0xDC => Some(0x00DC), // #LATIN CAPITAL LETTER U WITH DIAERESIS
        0xDD => Some(0x00DD), // #LATIN CAPITAL LETTER Y WITH ACUTE
        0xDE => Some(0x00DE), // #LATIN CAPITAL LETTER THORN
        0xDF => Some(0x00DF), // #LATIN SMALL LETTER SHARP S
        0xE0 => Some(0x00E0), // #LATIN SMALL LETTER A WITH GRAVE
        0xE1 => Some(0x00E1), // #LATIN SMALL LETTER A WITH ACUTE
        0xE2 => Some(0x00E2), // #LATIN SMALL LETTER A WITH CIRCUMFLEX
        0xE3 => Some(0x00E3), // #LATIN SMALL LETTER A WITH TILDE
        0xE4 => Some(0x00E4), // #LATIN SMALL LETTER A WITH DIAERESIS
        0xE5 => Some(0x00E5), // #LATIN SMALL LETTER A WITH RING ABOVE
        0xE6 => Some(0x00E6), // #LATIN SMALL LETTER AE
        0xE7 => Some(0x00E7), // #LATIN SMALL LETTER C WITH CEDILLA
        0xE8 => Some(0x00E8), // #LATIN SMALL LETTER E WITH GRAVE
        0xE9 => Some(0x00E9), // #LATIN SMALL LETTER E WITH ACUTE
        0xEA => Some(0x00EA), // #LATIN SMALL LETTER E WITH CIRCUMFLEX
        0xEB => Some(0x00EB), // #LATIN SMALL LETTER E WITH DIAERESIS
        0xEC => Some(0x00EC), // #LATIN SMALL LETTER I WITH GRAVE
        0xED => Some(0x00ED), // #LATIN SMALL LETTER I WITH ACUTE
        0xEE => Some(0x00EE), // #LATIN SMALL LETTER I WITH CIRCUMFLEX
        0xEF => Some(0x00EF), // #LATIN SMALL LETTER I WITH DIAERESIS
        0xF0 => Some(0x00F0), // #LATIN SMALL LETTER ETH
        0xF1 => Some(0x00F1), // #LATIN SMALL LETTER N WITH TILDE
        0xF2 => Some(0x00F2), // #LATIN SMALL LETTER O WITH GRAVE
        0xF3 => Some(0x00F3), // #LATIN SMALL LETTER O WITH ACUTE
        0xF4 => Some(0x00F4), // #LATIN SMALL LETTER O WITH CIRCUMFLEX
        0xF5 => Some(0x00F5), // #LATIN SMALL LETTER O WITH TILDE
        0xF6 => Some(0x00F6), // #LATIN SMALL LETTER O WITH DIAERESIS
        0xF7 => Some(0x00F7), // #DIVISION SIGN
        0xF8 => Some(0x00F8), // #LATIN SMALL LETTER O WITH STROKE
        0xF9 => Some(0x00F9), // #LATIN SMALL LETTER U WITH GRAVE
        0xFA => Some(0x00FA), // #LATIN SMALL LETTER U WITH ACUTE
        0xFB => Some(0x00FB), // #LATIN SMALL LETTER U WITH CIRCUMFLEX
        0xFC => Some(0x00FC), // #LATIN SMALL LETTER U WITH DIAERESIS
        0xFD => Some(0x00FD), // #LATIN SMALL LETTER Y WITH ACUTE
        0xFE => Some(0x00FE), // #LATIN SMALL LETTER THORN
        0xFF => Some(0x00FF), // #LATIN SMALL LETTER Y WITH DIAERESIS
        _ => None,
    }
}
