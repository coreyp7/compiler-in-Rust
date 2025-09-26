mod token;
mod token_type;

// public api imports for this module
pub use token::Token;
pub use token_type::TokenType;
//

use token_type::TokenType::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::FromStr;

// ============================================================================
// Data Structures and Enums
// ============================================================================

#[derive(Debug)]
enum TokenMatch {
    Single(TokenType),
    Double(char, TokenType), // (next_char, token_if_matched)
    Comment,                 // Special case for comments
}

// Note: The Tokenizer struct was previously defined here but is not currently used.
// It could be useful for future refactoring to make the tokenizer stateful.

// ============================================================================
// Utility Functions
// ============================================================================

fn create_token(
    token_type_param: TokenType,
    text_param: String,
    line_number: u32,
    curr_byte_index_in_line: usize,
) -> Token {
    Token {
        token_type: token_type_param,
        lexeme: text_param,
        line_number: line_number,
        col_number: curr_byte_index_in_line + 1,
    }
}

fn get_char_token_map() -> HashMap<char, TokenMatch> {
    use TokenMatch::*;
    //use TokenType::*;

    [
        ('+', Single(Plus)),
        ('*', Single(Asterisk)),
        (':', Single(Colon)),
        ('(', Single(LeftParen)),
        (')', Single(RightParen)),
        (',', Single(Comma)),
        ('-', Double('>', Arrow)),
        ('/', Comment),
        ('=', Double('=', EqualEqual)),
        ('<', Double('=', LessThanEqualTo)),
        ('>', Double('=', GreaterThanEqualTo)),
        ('!', Double('=', NotEqual)),
        ('&', Double('&', DoubleAmpersand)),
        ('|', Double('|', DoubleBar)),
    ]
    .into_iter()
    .collect()
}

fn get_single_char_fallback(ch: char) -> TokenType {
    match ch {
        '-' => TokenType::Minus,
        '=' => TokenType::Equal,
        '<' => TokenType::LessThan,
        '>' => TokenType::GreaterThan,
        '!' => TokenType::Bang,
        '/' => TokenType::Slash,
        _ => TokenType::UnsupportedSymbolError,
    }
}

// ============================================================================
// Core Tokenization Logic
// ============================================================================

/**
Can either be:
- a String
- a Number
- an Identifier

Specify which kind of token you're looking for the end of.

Given
- string buffer
- start of token

Will return -> (Token, new position in string buffer)
*/
fn create_token_from_lexume(
    line_bytes: &[u8],
    token_start: usize,
    token_type: TokenType,
    line_number: u32,
) -> (Token, usize) {
    let mut str_byte_buffer = token_start;
    let mut end_of_string_idx: Option<usize> = None;
    let mut string_content: String = String::new();

    while str_byte_buffer < line_bytes.len() {
        let curr_char = line_bytes[str_byte_buffer] as char;
        // 3 cases: String, Number, or Identifier
        // this could be organized better I think but leaving as is for now,
        // since its clear to me how this is organized
        match token_type {
            TokenType::Number => {
                if curr_char.is_numeric() == false {
                    end_of_string_idx = Some(str_byte_buffer - 1);
                    str_byte_buffer = line_bytes.len();
                } else if str_byte_buffer == line_bytes.len() - 1 {
                    string_content.push(curr_char);
                    end_of_string_idx = Some(str_byte_buffer);
                    str_byte_buffer = line_bytes.len();
                } else {
                    string_content.push(curr_char);
                }
            }
            TokenType::Str => {
                if curr_char == '"' {
                    end_of_string_idx = Some(str_byte_buffer);
                    str_byte_buffer = line_bytes.len();
                } else {
                    string_content.push(curr_char);
                }
            }
            TokenType::Identity => {
                /*
                Identities follow the same parsing rules as keywords.
                So, they are both processed in here, and then we figure out
                later if this token is a keyword instead of an Identity.
                */
                if curr_char.is_alphabetic() == false {
                    end_of_string_idx = Some(str_byte_buffer - 1);
                    str_byte_buffer = line_bytes.len();
                } else if str_byte_buffer == line_bytes.len() - 1 {
                    string_content.push(curr_char);
                    end_of_string_idx = Some(str_byte_buffer);
                    str_byte_buffer = line_bytes.len();
                } else {
                    string_content.push(curr_char);
                }
            }
            _ => (),
        }

        str_byte_buffer += 1;
    }

    let mut new_curr_byte_idx: usize = 0;
    let mut token = create_token(
        TokenType::UnsupportedSymbolError,
        String::from(""),
        line_number,
        0,
    );

    match end_of_string_idx {
        None => {
            // TODO: this needs to make a bigger deal out of this.
            // This compilation should end here; unable to tokenize.
            ()
        }
        Some(new_curr_idx) => {
            token = create_token(token_type, string_content, line_number, token_start);
            new_curr_byte_idx = new_curr_idx + 1;
        }
    };

    if token.token_type == TokenType::Identity {
        // check if this is a keyword, and change it accordingly
        // create function that returns the tokentype given a string keyword.
        // if it returns Some(type) then update the tokentype.
        // else None, then keep this as an Identity, since it doesn't match any keyword.
        let f = TokenType::from_str(&token.lexeme);
        match f {
            Ok(token_type) => token.token_type = token_type,
            _ => {}
        }
    }
    (token, new_curr_byte_idx)
}

fn create_token_at_byte_in_line(
    line_bytes: &[u8],
    curr: char,
    next: Option<char>,
    line_number: u32,
    curr_byte_index_in_line: usize,
) -> (Token, bool, usize) {
    let token_map = get_char_token_map();

    /**
     * TODO: need to split up this function and
     * change the name to be less esoteric.
     *
     * top if block handles matching symbols only
     * bottom else block handles strings, keywords, and numbers.
     */
    if let Some(token_match) = token_map.get(&curr) {
        match token_match {
            TokenMatch::Single(token_type) => (
                create_token(
                    *token_type,
                    String::from(curr),
                    line_number,
                    curr_byte_index_in_line,
                ),
                false,
                curr_byte_index_in_line + 1,
            ),
            TokenMatch::Double(expected_next, double_token_type) => {
                if matches!(next, Some(x) if x == *expected_next) {
                    let double_char = format!("{}{}", curr, expected_next);
                    let token = create_token(
                        *double_token_type,
                        double_char,
                        line_number,
                        curr_byte_index_in_line,
                    );

                    // We need to increment by 2 since we consumed two characters
                    return (token, false, curr_byte_index_in_line + 2);
                } else {
                    let fallback_type = get_single_char_fallback(curr);
                    (
                        create_token(
                            fallback_type,
                            String::from(curr),
                            line_number,
                            curr_byte_index_in_line,
                        ),
                        false,
                        curr_byte_index_in_line + 1,
                    )
                }
            }
            TokenMatch::Comment => {
                if matches!(next, Some(x) if x == '!') {
                    // Return a dummy token and signal to skip rest of line
                    (
                        create_token(
                            TokenType::UnsupportedSymbolError,
                            String::new(),
                            line_number,
                            curr_byte_index_in_line,
                        ),
                        true,
                        curr_byte_index_in_line + 2,
                    )
                } else {
                    (
                        create_token(
                            TokenType::Slash,
                            String::from(curr),
                            line_number,
                            curr_byte_index_in_line,
                        ),
                        false,
                        curr_byte_index_in_line + 1,
                    )
                }
            }
        }
    } else {
        // Handle strings, keywords, and numbers.
        // They are handled slightly differently depending on which, so we
        // condition the data we pass into create_token_from_lexume.
        let token_type = match curr {
            '"' => Some(TokenType::Str),
            x if x.is_numeric() => Some(TokenType::Number),
            x if x.is_alphabetic() => Some(TokenType::Identity),
            _ => None,
        };

        if let Some(token_type) = token_type {
            let start_index = if token_type == TokenType::Str {
                curr_byte_index_in_line + 1 // Skip opening quote
            } else {
                curr_byte_index_in_line
            };

            let result = create_token_from_lexume(line_bytes, start_index, token_type, line_number);
            (result.0, false, result.1)
        } else {
            /*
             * TODO: Spaces should be handled more explicitly. Something else
             * could go wrong here and we wouldn't know.
             */
            (
                create_token(
                    TokenType::UnsupportedSymbolError,
                    String::from(curr),
                    line_number,
                    curr_byte_index_in_line,
                ),
                false,
                curr_byte_index_in_line + 1,
            )
        }
    }
}

// ============================================================================
// Main Tokenization Functions
// ============================================================================

fn tokenize_line(line: String, line_number: u32) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let line_bytes: &[u8] = line.as_bytes();
    //let mut curr_byte_index = 0;
    let mut curr_byte_index_in_line: usize = 0;

    while curr_byte_index_in_line < line_bytes.len() {
        let curr = line_bytes[curr_byte_index_in_line];
        let curr = curr as char;
        let mut next: Option<char> = None;
        if curr_byte_index_in_line < line_bytes.len() - 1 {
            next = Some(line_bytes[curr_byte_index_in_line + 1] as char);
        }

        let (token, skip_rest_of_line, new_index) = create_token_at_byte_in_line(
            &line_bytes,
            curr,
            next,
            line_number,
            curr_byte_index_in_line,
        );

        curr_byte_index_in_line = new_index;

        if token.token_type != TokenType::UnsupportedSymbolError {
            tokens.push(token);
        }

        if skip_rest_of_line {
            // This is a comment; we're skipping the rest of the line.
            curr_byte_index_in_line = line_bytes.len();
        }
    }

    // End tokens list with new line, since this is the end of the line.
    // If you want to include these for good formatting, add a flag or something
    // to allow that here. TODO
    /*
    tokens.push(create_token(
        TokenType::Newline,
        String::new(),
        line_number,
        line.len(),
    ));
    */

    tokens
}

/**
 * TODO: check validity of file just in case.
 */
pub fn tokenize_file(src_file: &mut File) -> Vec<Token> {
    let reader = BufReader::new(src_file);
    let mut token_vec: Vec<Token> = Vec::new();
    let mut line_number = 1;

    for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
            let mut tokens = tokenize_line(line_str, line_number);
            token_vec.append(&mut tokens);
            line_number += 1;
        }
    }
    token_vec.push(create_token(TokenType::EOF, String::new(), line_number, 0));

    token_vec
}
