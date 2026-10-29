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


// -- Lexical Analyzer

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

/// Trait for a recursive descent Syntax Analyzer 
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
    fn parse_list(&mut self);           
    fn parse_list_items(&mut self);     
    fn parse_inner_list(&mut self);     
    fn parse_link(&mut self);          
    fn parse_newline(&mut self);        
    fn parse_text(&mut self);           
}

pub struct SimpleLexicalAnalyzer {
    input: Vec<char>,
    position: usize,
    current_build: String,
    pub tokens: Vec<String>,
    pub start_tags: Vec<String>,
    pub end_tags: Vec<String>,
    pub tag_names: Vec<String>,
    pub statements: Vec<String>,

}

impl SimpleLexicalAnalyzer {
    pub fn new (source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_build: String::new(),
            tokens: Vec::new(),
            start_tags: vec!["#HAI".into(), "#OBTW".into(), "#MAEK".into(), "#GIMMEH".into()],
            end_tags: vec!["#KBYE".into(), "#TLDR".into(), "#OIC".into(), "#MKAY".into()],
            tag_names: vec!["HEAD".into(), "TITLE".into(), "PARAGRAF".into(), "BOLD".into(), "LIST".into(), "ITEM".into(), "LINX".into()],
            statements: vec!["#NEWLINE".into(), "#IHAZ".into(), "#ITIZ".into(), "VAR".into()],
        }
    }

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char();
            if c == '\0' {
                break;
            }
            if c.is_whitespace() {
                if !self.current_build.is_empty() {
                    self.tokens.push(std::mem::take(&mut self.current_build));
                }
            } else {
                self.add_char(c);
            }
        }
        if !self.current_build.is_empty() {
            self.tokens.push(std::mem::take(&mut self.current_build));
        }
        self.tokens.reverse();
    }
}

impl LexicalAnalyzer for SimpleLexicalAnalyzer {
    fn get_char(&mut self) -> char {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            c
        } else {
            '\0'
        }
    }

    fn add_char(&mut self, c: char) {
        self.current_build.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        self.start_tags.iter().any(|s_tag| s_tag == s) ||
        self.end_tags.iter().any(|e_tag| e_tag == s) ||
        self.tag_names.iter().any(|n_tag| n_tag == s) ||
        self.statements.iter().any(|stmt| stmt == s)
    }
}

// -- Syntax Analyzer

pub struct LolcodeCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
}

impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
        }
    }

    fn start(&mut self) {
        let candidate = self.lexer.tokens.pop().unwrap_or_default();
        if self.lexer.lookup(&candidate) {
            self.current_tok = candidate;
        } else if !candidate.is_empty() {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate);
            std::process::exit(1);
        } else {
            eprintln!("User error: The provided sentence is empty.");
            std::process::exit(1);
        }
    }

    #[inline] fn is_start_tag(&self, s: &str) -> bool { self.lexer.start_tags.iter().any(|s_tag| s_tag == s) }
    #[inline] fn is_end_tag(&self, s: &str) -> bool { self.lexer.end_tags.inter().any(|e_tag| e_tag == s) }
    #[inline] fn is_tag_name(&self, s: &str) -> bool { self.lexer.tag_names.inter().any()(|n_tag| n_tag == s) }
    #[inline] fn is_statement(&self, s: &str) -> bool { self.lexer.statements.inter().any()(|stmt| stmt == s)}

    fn parse_lolcode(&mut self){
    }

    fn parse_head(&mut self){
    }

    fn parse_title(&mut self){
    }

    fn parse_comment(&mut self){
    }

    fn parse_body(&mut self){
    }

    fn parse_paragraph(&mut self){
    }

    fn parse_inner_paragraph(&mut self){        
    }

    fn parse_inner_text(&mut self){
    }

    fn parse_variable_define(&mut self){
    }

    fn parse_variable_use(&mut self){
    }

    fn parse_bold(&mut self){
    }

    fn parse_list(&mut self){
    }

    fn parse_list_items(&mut self){
    }

    fn parse_inner_list(&mut self){
    }

    fn parse_link(&mut self){
    }

    fn parse_newline(&mut self){
    }

    fn parse_text(&mut self){
    }

}


impl Compiler for LolcodeCompiler {
    fn compile(&mut self, source: &str) {
        self.lexer = SimpleLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.start();
    }

    fn next_token(&mut self) -> String {
        let candidate = self.lexer.tokens.pop().unwrap_or_default();
        if self.lexer.lookup(&candidate) {
            self.current_tok = candidate.clone();
            candidate
        } else if self.lexer.tokens.is_empty() {
            self.current_tok.clear();
            String::new()
        } else {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate);
            std::process::exit(1);
        }
    }

    fn parse(&mut self) {
        self.lolspeak();
        if !self.lexer.tokens.is_empty() || !self.current_tok.is_empty() {
            eprintln!("Syntax error: Additional tokens found after the sentence.");
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String { self.current_tok.clone() }
    fn set_current_token(&mut self, tok: String) { self.current_tok = tok; }
}



// -- Main
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let sentence = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    let mut compiler = LolcodeCompiler::new();
    compiler.compile(&sentence);
    compiler.parse();

}
