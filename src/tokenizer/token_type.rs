use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum TokenType {
    EOF = 0,
    Newline,
    Number,
    Identity,
    Str,
    Boolean,
    True,
    False,
    // Keywords
    Label = 100, //unused
    //NumberType, // for declaring variable 'Number'
    VarDeclaration,
    FunctionDeclaration,
    UpdateKeyword, // assigning to variables
    Goto,
    Print,
    Println,
    Input,
    Let,
    If,
    Then,
    Else,
    EndIf,
    While,
    Do,
    EndWhile,
    Return,
    Returns, // used in function declarations
    LeftParen,
    RightParen,
    Comma,
    Arrow,
    EndFunction,
    // Operators
    Equal = 200,
    Plus,
    Minus,
    Asterisk,
    Slash,
    EqualEqual, // 205
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo, // 210
    DoubleAmpersand,
    DoubleBar,
    Bang,
    Colon,
    Semicolon,
    UnsupportedSymbolError = 900,
    // Won't get through to the parser, just for processing in here.
    Space,
}

/**
 * This allows for easy matching of a keyword (as a String) to its
 * TokenType counterpart.
*/
impl FromStr for TokenType {
    type Err = ();

    fn from_str(input: &str) -> Result<TokenType, Self::Err> {
        match input {
            "label" => Ok(TokenType::Label),
            "goto" => Ok(TokenType::Goto),
            "print" => Ok(TokenType::Print),
            "println" => Ok(TokenType::Println),
            "input" => Ok(TokenType::Input),
            "let" => Ok(TokenType::Let),
            "if" => Ok(TokenType::If),
            "then" => Ok(TokenType::Then),
            "endIf" => Ok(TokenType::EndIf),
            "while" => Ok(TokenType::While),
            "do" => Ok(TokenType::Do),
            "endWhile" => Ok(TokenType::EndWhile),
            "Number" | "String" | "Boolean" => Ok(TokenType::VarDeclaration),
            "update" => Ok(TokenType::UpdateKeyword),
            "function" => Ok(TokenType::FunctionDeclaration),
            "return" => Ok(TokenType::Return),
            "returns" => Ok(TokenType::Returns),
            "endFunction" => Ok(TokenType::EndFunction),
            "else" => Ok(TokenType::Else),
            "Boolean" => Ok(TokenType::Boolean),
            "true" => Ok(TokenType::True),
            "false" => Ok(TokenType::False),
            "and" => Ok(TokenType::DoubleAmpersand),
            "or" => Ok(TokenType::DoubleBar),
            _ => Err(()),
        }
    }
}

// TODO: this may be kindof gross having to edit both here and above.
// Look into if any workaround available.
impl TokenType {
    /// Converts a TokenType to its string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            TokenType::EOF => "EOF",
            TokenType::Newline => "\\n",
            TokenType::Number => "Number",
            TokenType::Identity => "Identity",
            TokenType::Str => "String",
            // Keywords
            TokenType::Label => "label",
            TokenType::VarDeclaration => "VarDeclaration",
            TokenType::UpdateKeyword => "update",
            TokenType::FunctionDeclaration => "FunctionDeclaration",
            TokenType::Goto => "goto",
            TokenType::Print => "print",
            TokenType::Println => "println",
            TokenType::Input => "input",
            TokenType::Let => "let",
            TokenType::If => "if",
            TokenType::Then => "then",
            TokenType::Else => "else",
            TokenType::EndIf => "endIf",
            TokenType::While => "while",
            TokenType::Do => "do",
            TokenType::EndWhile => "endWhile",
            // Operators
            TokenType::Equal => "=",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Asterisk => "*",
            TokenType::Slash => "/",
            TokenType::EqualEqual => "==",
            TokenType::NotEqual => "!=",
            TokenType::LessThan => "<",
            TokenType::LessThanEqualTo => "<=",
            TokenType::GreaterThan => ">",
            TokenType::GreaterThanEqualTo => ">=",
            TokenType::DoubleAmpersand => "&&",
            TokenType::DoubleBar => "||",
            TokenType::Bang => "!",
            TokenType::Colon => ":",
            TokenType::UnsupportedSymbolError => "UnsupportedSymbol",
            TokenType::Space => " ",
            TokenType::Return => "return",
            TokenType::Returns => "returns",
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::Comma => ",",
            TokenType::Arrow => "->",
            TokenType::EndFunction => "endFunction",
            TokenType::Semicolon => ";",
            TokenType::Boolean => "bool",
            TokenType::True => "true",
            TokenType::False => "false",
        }
    }
}
