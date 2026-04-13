// Project 1 LOLCODE 

fn main() {
     // Read the source file from command line argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = std::fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", filename, e);
        std::process::exit(1);
    });

    let mut compiler = LOLCompiler::new(&source);

    // Run the lexer
    compiler.compile(&source);

    // Print all tokens for testing
    println!("{:<5} {:<15} {:<20} {}", "Line", "TokenType", "Value", "---");
    println!("{}", "-".repeat(55));
    for tok in &compiler.tokens {
        println!("{:<5} {:<15?} {:<20}", tok.line, tok.token_type, tok.value);
    }
    println!("{}", "-".repeat(55));
    println!("Total tokens: {}", compiler.tokens.len());
}


/// Trait for a simple lolcompiler front-end.
/// Errors should cause immediate exit inside the implementation.
pub trait Compiler {
/// Begin the compilation process (entry point).
fn compile(&mut self, source: &str);
/// Get the next token from the lexical analyzer.
fn next_token(&mut self) -> String;
/// Run the syntax analyzer starting from <lolcode>.
fn parse(&mut self);
/// Get the current token being processed.
fn current_token(&self) -> String;
/// Set the current token (typically used internally).
fn set_current_token(&mut self, tok: String);
}

pub struct LOLCompiler {
    source: Vec<char>,
    pos: usize,
    current_token: Token,
    tokens: Vec<Token>,
    output: String,
    line: usize,
    scope_stack: Vec<std::collections::HashMap<String, String>>
}

impl LOLCompiler {
    pub fn new(source: &str) -> Self {
        let mut src = source.to_string();
        src.push('\n'); // Ensure source ends with newline for lexer
        LOLCompiler {
            source: src.chars().collect(),
            pos: 0,
            current_token: Token {
                token_type: TokenType::Eof,
                value: String::new(),
                line: 1,
            },
            tokens: Vec::new(),
            output: String::new(),
            line: 1,
            scope_stack: Vec::new(),
        }
    }
}

impl Compiler for LOLCompiler {
    fn next_token(&mut self) -> String {
        let mut state = LexerState::Start;
        self.current_token.value.clear();

        loop {
            // peek at current char without consuming
            if self.pos >= self.source.len() {
                self.current_token.token_type = TokenType::Eof;
                return String::new();
            }

            let c = self.source[self.pos];

            match state {
                LexerState::Start => {
                    // skip whitespace
                    if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                        self.get_char();
                    } else if c == '#' {
                        self.current_token.line = self.line;
                        self.get_char(); // consume '#'
                        state = LexerState::InHash;
                    } else if c.is_alphanumeric() || ",.\":?!/%-".contains(c) {
                        self.current_token.line = self.line;
                        state = LexerState::InWord;
                    } else {
                        eprintln!("Error: Unexpected character '{}' at line {}", c, self.line);
                        std::process::exit(1);
                    }
                }

                LexerState::InHash => {
                    if c.is_alphabetic() && self.pos < self.source.len() {
                        let ch = self.get_char();
                        self.add_char(ch);
                    } else {
                        // we have the keyword after #, look it up
                        let word = self.current_token.value.to_lowercase();
                        if !self.lookup(&word) {
                            eprintln!(
                                "Error: Unknown annotation '#{}'  at line {}",
                                self.current_token.value, self.line
                            );
                            std::process::exit(1);
                        }
                        self.current_token.token_type = match word.as_str() {
                            "hai"      => TokenType::Hai,
                            "kbye"     => TokenType::Kbye,
                            "obtw"     => TokenType::Obtw,
                            "tldr"     => TokenType::Tldr,
                            "maek"     => TokenType::Maek,
                            "oic"      => TokenType::Oic,
                            "gimmeh"   => TokenType::Gimmeh,
                            "mkay"     => TokenType::Mkay,
                            "ihaz"     => TokenType::Ihaz,
                            "itiz"     => TokenType::Itiz,
                            "lemmesee" => TokenType::Lemmesee,
                            "newline"  => TokenType::Newline,
                            _ => {
                                eprintln!(
                                    "Error: Unknown annotation '#{}'  at line {}",
                                    self.current_token.value, self.line
                                );
                                std::process::exit(1);
                            }
                        };
                        self.current_token.value = format!("#{}", word);
                        state = LexerState::Done;
                    }
                }

                LexerState::InWord => {
                    if (c.is_alphanumeric() || ",.\":?!/%-_=".contains(c)) && self.pos < self.source.len() {
                        let ch = self.get_char();
                        self.add_char(ch);
                    } else {
                        let word = self.current_token.value.to_lowercase();
                        self.current_token.token_type = match word.as_str() {
                            "head"     => TokenType::Head,
                            "title"    => TokenType::Title,
                            "paragraf" => TokenType::Paragraf,
                            "bold"     => TokenType::Bold,
                            "italics"  => TokenType::Italics,
                            "list"     => TokenType::List,
                            "item"     => TokenType::Item,
                            "linx"     => TokenType::Linx,
                            _          => TokenType::Text,
                        };
                        state = LexerState::Done;
                    }
                }

                LexerState::Done => {
                    return self.current_token.value.clone();
                }

                LexerState::Error => {
                    eprintln!("Error: Lexer error at line {}", self.line);
                    std::process::exit(1);
                }
            }
        }
    }

    fn compile(&mut self, _source: &str) {
        loop {
            self.next_token();
            let tok = self.current_token.clone();
            if tok.token_type == TokenType::Eof {
                break;
            }
            self.tokens.push(tok);
        }
    }

    fn current_token(&self) -> String {
        self.current_token.value.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current_token.value = tok;
    }

    fn parse(&mut self) {
        todo!()
    }
}

/// OPTION 1 - Trait for a recursive descent Syntax Analyzer
/// over Vec<String>. Each function parses a nonterminal in
/// the grammar. On error: exit immediately.
pub trait SyntaxAnalyzer {
fn parse_lolcode(&mut self);
fn parse_head(&mut self);
fn parse_title(&mut self);
fn parse_comment(&mut self);
fn parse_body(&mut self);
fn parse_paragraph(&mut self);
fn parse_inner_paragraph(&mut self);
fn parse_inner_text(&mut self);
fn parse_variable_define(&mut self);
fn parse_variable_use(&mut self);
fn parse_bold(&mut self);
fn parse_italics(&mut self);
fn parse_list(&mut self);
fn parse_list_items(&mut self);
fn parse_inner_list(&mut self);
fn parse_link(&mut self);
fn parse_newline(&mut self);
fn parse_text(&mut self);
}

impl SyntaxAnalyzer for LOLCompiler {
    fn parse_lolcode(&mut self) {
        
    }

    fn parse_head(&mut self) {
        
    }

    fn parse_title(&mut self) {
        
    }

    fn parse_comment(&mut self) {
        
    }

    fn parse_body(&mut self) {
        
    }

    fn parse_paragraph(&mut self) {
        
    }

    fn parse_inner_paragraph(&mut self) {
        
    }

    fn parse_inner_text(&mut self) {
        
    }

    fn parse_variable_define(&mut self) {
        
    }

    fn parse_variable_use(&mut self) {
        
    }

    fn parse_bold(&mut self) {
        
    }

    fn parse_italics(&mut self) {
        
    }

    fn parse_list(&mut self) {
        
    }

    fn parse_list_items(&mut self) {
        
    }

    fn parse_inner_list(&mut self) {
        
    }

    fn parse_link(&mut self) {
        
    }

    fn parse_newline(&mut self) {
        
    }

    fn parse_text(&mut self) {
        
    }
}

/// Trait for a simple lexical analyzer.
/// Implements a character-by-character analysis
/// from a state machine design.
pub trait LexicalAnalyzer {
/// Return the next character from the input.
/// If input is exhausted, should terminate the program.
fn get_char(&mut self) -> char;
/// Add a character to the current potential token.
fn add_char(&mut self, c: char);
/// Lookup a potential token to determine if it is valid.
/// Returns true if a valid token/lexeme, false otherwise.
fn lookup(&self, s: &str) -> bool;
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Hai, Kbye, Obtw, Tldr, Maek, Oic,
    Gimmeh, Mkay, Head, Title, Paragraf,
    Bold, Italics, List, Item, Newline,
    Linx, Ihaz, Itiz, Lemmesee,
    Varname, Varvalue, Text, Address,
    Eof
}
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize
}

pub enum LexerState {
    Start,        // beginning, skip whitespace
    InHash,       // just saw '#', building annotation
    InWord,       // building a plain word (text, varname, address)
    Done,         // token is complete
    Error,        // invalid character or unknown annotation
}

impl LexicalAnalyzer for LOLCompiler {
    fn get_char(&mut self) -> char {
        if self.pos >= self.source.len() {
            eprintln!("Error: Unexpected end of input at line{}", self.line);
            std::process::exit(1);
        }
        let c = self.source[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        c
    }

    fn add_char(&mut self, c: char) {
        self.current_token.value.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        matches!(s.to_lowercase().as_str(),
            "hai" | "kbye" | "obtw" | "tldr" | "maek" | "oic" |
            "gimmeh" | "mkay" | "head" | "title" | "paragraf" |
            "bold" | "italics" | "list" | "item" | "newline" |
            "linx" | "ihaz" | "itiz" | "lemmesee"
        )
    }
}