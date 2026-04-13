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

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: TokenType) {
        if self.tokens[self.pos].token_type != expected {
            eprintln!(
                "Error: Expected token {:?} but found {:?} at line {}",
                expected, self.tokens[self.pos].token_type, self.tokens[self.pos].line
            );
            std::process::exit(1);
        }
        self.advance();
    }

    // define a variable in the current scope
    fn define_variable(&mut self, name: String, value: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, value);
        }
    }

    fn resolve_variable(&self, name: &str) -> Option<&String> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

}

impl SyntaxAnalyzer for LOLCompiler { 

    fn parse_lolcode(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); //global scope
        self.expect(TokenType::Hai);
        // parse optional comment and head sections
        if self.peek().token_type == TokenType::Obtw {
            self.parse_comment();
        }
        if self.peek().token_type == TokenType::Maek {
            self.parse_head();
        }
        // parse body
        self.parse_body();
        //file end must be Kbye
        self.expect(TokenType::Kbye);
        self.scope_stack.pop(); // exit global scope
    }

    fn parse_comment(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); //comment scope
        self.expect(TokenType::Obtw);
        // consume all text until #TLDR
        while self.peek().token_type != TokenType::Tldr {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed comment, expected #TLDR at line {}", 
                    self.peek().line);
                std::process::exit(1);
            }
            self.advance();
        }
        self.expect(TokenType::Tldr);
        self.scope_stack.pop(); // exit comment scope
    }

    fn parse_head(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); //head scope
        self.expect(TokenType::Maek);
        self.expect(TokenType::Head);
        // optional comment
        if self.peek().token_type == TokenType::Obtw {
            self.parse_comment();
        }
        // optional title
        if self.peek().token_type == TokenType::Gimmeh {
            self.parse_title();
        }
        self.expect(TokenType::Mkay);
        self.scope_stack.pop(); // exit head scope
    }

    fn parse_title(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new());
        self.expect(TokenType::Gimmeh);
        self.expect(TokenType::Title);
        // consume text until #OIC
        while self.peek().token_type != TokenType::Oic {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed title, expected #OIC at line {}",
                    self.peek().line);
                std::process::exit(1);
            }
            self.advance();
        }
        self.expect(TokenType::Oic);
        self.scope_stack.pop();
    }

    fn parse_body(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); //body scope
        while self.peek().token_type != TokenType::Kbye {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Expected #KBYE at line {}", self.peek().line);
                std::process::exit(1);
            }
            match self.peek().token_type {
                TokenType::Obtw     => self.parse_comment(),
                TokenType::Gimmeh   => self.parse_inner_text(),
                TokenType::Ihaz     => self.parse_variable_define(),
                TokenType::Lemmesee => self.parse_variable_use(),
                TokenType::Newline  => self.parse_newline(),
                TokenType::Text     => self.parse_text(),
                TokenType::Maek => {
                    // check paragraph or list
                    if self.tokens[self.pos + 1].token_type == TokenType::Paragraf {
                        self.parse_paragraph()
                    } else {
                        self.parse_list()
                    }
                },
                _ => {
                    eprintln!("Error: Unexpected token '{}' at line {}",
                        self.peek().value, self.peek().line);
                    std::process::exit(1);
                }
            }
        }
        self.scope_stack.pop(); // exit body scope
    }

    fn parse_paragraph(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); //paragraph scope
        self.expect(TokenType::Maek);
        self.expect(TokenType::Paragraf);
        // optional variable definition must be first
        if self.peek().token_type == TokenType::Ihaz {
            self.parse_variable_define();
        }
        // parse inner content until #MKAY
        self.parse_inner_paragraph();
        self.expect(TokenType::Mkay);
        self.scope_stack.pop(); // exit paragraph scope
    }

    fn parse_inner_paragraph(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new());
        while self.peek().token_type != TokenType::Mkay {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed paragraph, expected #MKAY at line {}",
                    self.peek().line);
                std::process::exit(1);
            }
            match self.peek().token_type {
                TokenType::Obtw     => self.parse_comment(),
                TokenType::Gimmeh   => self.parse_inner_text(),
                TokenType::Ihaz     => self.parse_variable_define(),
                TokenType::Lemmesee => self.parse_variable_use(),
                TokenType::Newline  => self.parse_newline(),
                TokenType::Text     => self.parse_text(),
                TokenType::Maek => self.parse_list(),
                _ => {
                    eprintln!("Error: Unexpected token '{}' inside paragraph at line {}",
                        self.peek().value, self.peek().line);
                    std::process::exit(1);
                }
            }
        }
        self.scope_stack.pop(); // exit paragraph scope
    }

    fn parse_inner_text(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); // inner text scope
        self.expect(TokenType::Gimmeh);
        match self.peek().token_type {
            TokenType::Bold    => self.parse_bold(),
            TokenType::Italics => self.parse_italics(),
            TokenType::Linx    => self.parse_link(),
            TokenType::Newline => self.parse_newline(),
            _ => {
                eprintln!("Error: Unexpected token '{}' after #GIMMEH at line {}",
                    self.peek().value, self.peek().line);
                std::process::exit(1);
            }
        }
        self.scope_stack.pop(); // exit inner text scope
    }

    fn parse_bold(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new());
        self.expect(TokenType::Bold);
        // optional variable definition must be first
        if self.peek().token_type == TokenType::Ihaz {
            self.parse_variable_define();
        }
        // consume content until #OIC
        while self.peek().token_type != TokenType::Oic {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed bold, expected #OIC at line {}",
                    self.peek().line);
                std::process::exit(1);
            }
            match self.peek().token_type {
                TokenType::Text     => self.parse_text(),
                TokenType::Lemmesee => self.parse_variable_use(),
                _ => {
                    eprintln!("Error: Unexpected token '{}' inside bold at line {}",
                        self.peek().value, self.peek().line);
                    std::process::exit(1);
                }
            }
        }
        self.expect(TokenType::Oic);
        self.scope_stack.pop(); // exit bold scope
    }

    fn parse_italics(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); // italics scope
        self.expect(TokenType::Italics);
        // optional variable definition must be first
        if self.peek().token_type == TokenType::Ihaz {
            self.parse_variable_define();
        }
        // consume content until #OIC
        while self.peek().token_type != TokenType::Oic {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed italics, expected #OIC at line {}",
                    self.peek().line);
                std::process::exit(1);
            }
            match self.peek().token_type {
                TokenType::Text     => self.parse_text(),
                TokenType::Lemmesee => self.parse_variable_use(),
                _ => {
                    eprintln!("Error: Unexpected token '{}' inside italics at line {}",
                        self.peek().value, self.peek().line);
                    std::process::exit(1);
                }
            }
        }
        self.expect(TokenType::Oic);
        self.scope_stack.pop(); // exit italics scope
    }

    fn parse_list(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); // list scope
        self.expect(TokenType::Maek);
        self.expect(TokenType::List);
        // optional variable definition must be first
        if self.peek().token_type == TokenType::Ihaz {
            self.parse_variable_define();
        }
        // must have at least one item
        if self.peek().token_type != TokenType::Gimmeh {
            eprintln!("Error: Expected #GIMMEH ITEM inside list at line {}",
                self.peek().line);
            std::process::exit(1);
        }
        self.parse_list_items();
        self.expect(TokenType::Mkay);
        self.scope_stack.pop(); // exit list scope
    }

    fn parse_list_items(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); // list items scope
        // must have at least one item
        while self.peek().token_type == TokenType::Gimmeh {
            self.parse_inner_list();
        }
        self.scope_stack.pop(); // exit list items scope
    }

    fn parse_inner_list(&mut self) {
        self.scope_stack.push(std::collections::HashMap::new()); // list item scope
        self.expect(TokenType::Gimmeh);
        self.expect(TokenType::Item);
        // optional variable definition must be first
        if self.peek().token_type == TokenType::Ihaz {
            self.parse_variable_define();
        }
        // consume content until #OIC
        while self.peek().token_type != TokenType::Oic {
            if self.peek().token_type == TokenType::Eof {
                eprintln!("Error: Unclosed item, expected #OIC at line {}",
                    self.peek().line);
                std::process::exit(1);
            }
            match self.peek().token_type {
                TokenType::Text     => self.parse_text(),
                TokenType::Gimmeh   => self.parse_inner_text(),
                TokenType::Lemmesee => self.parse_variable_use(),
                _ => {
                    eprintln!("Error: Unexpected token '{}' inside item at line {}",
                        self.peek().value, self.peek().line);
                    std::process::exit(1);
                }
            }
        }
        self.expect(TokenType::Oic);
        self.scope_stack.pop(); // exit list item scope
    }

    fn parse_link(&mut self) {
        self.expect(TokenType::Linx);
        // expect an address (comes out as Text token from lexer)
        if self.peek().token_type != TokenType::Text {
            eprintln!("Error: Expected address after #GIMMEH LINX at line {}",
                self.peek().line);
            std::process::exit(1);
        }
        self.advance(); // consume the address
        self.expect(TokenType::Oic);
    }

    fn parse_newline(&mut self) {
        self.expect(TokenType::Newline);
    }

    fn parse_text(&mut self) {
        self.expect(TokenType::Text);
    }

    fn parse_variable_define(&mut self) {
        self.expect(TokenType::Ihaz);
        // variable name must be a single word (Text token)
        if self.peek().token_type != TokenType::Text {
            eprintln!("Error: Expected variable name after #IHAZ at line {}",
                self.peek().line);
            std::process::exit(1);
        }
        let name = self.tokens[self.pos].value.clone();
        self.advance(); // consume varname
        self.expect(TokenType::Itiz);
        // variable value must be a single word (Text token)
        if self.peek().token_type != TokenType::Text {
            eprintln!("Error: Expected variable value after #ITIZ at line {}",
                self.peek().line);
            std::process::exit(1);
        }
        let value = self.tokens[self.pos].value.clone();   
        self.advance(); // consume varvalue
        self.expect(TokenType::Mkay);
        // store in scope
        self.define_variable(name,value);
    }

    fn parse_variable_use(&mut self) {
        self.expect(TokenType::Lemmesee);
        // variable name must be a single word (Text token)
        if self.peek().token_type != TokenType::Text {
            eprintln!("Error: Expected variable name after #LEMMESEE at line {}",
                self.peek().line);
            std::process::exit(1);
        }
        let name = self.tokens[self.pos].value.clone();
        let line = self.tokens[self.pos].line;
        self.advance(); // consume varname
        self.expect(TokenType::Oic);

        if self.resolve_variable(&name).is_none() {
            eprintln!("Error: Variable '{}' used before definition at line {}",
                name, line);
            std::process::exit(1);
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
        self.parse(); // start syntax analysis after lexing all tokens
    }

    fn current_token(&self) -> String {
        self.current_token.value.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current_token.value = tok;
    }

    fn parse(&mut self) {
        self.pos = 0;
        self.parse_lolcode();
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