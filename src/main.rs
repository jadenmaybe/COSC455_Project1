use std::env;
use std::fs;
use std::process::{Command, ExitStatus};
use std::io;

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
        matches!(
            s.to_uppercase().as_str(),
            "#HAI" | "#KBYE" | "#OBTW" | "#TLDR" | "#MAEK" | "#OIC" | 
            "#GIMMEH" | "#MKAY" | "HEAD" | "TITLE" | "PARAGRAF" | 
            "BOLD" | "LIST" | "ITEM" | "#NEWLINE" | "LINX" | 
            "#IHAZ" | "#ITIZ" | "VAR"
        ) || !s.is_empty()
    }

}

// -- Syntax Analyzer

pub struct LolcodeCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    pub html_output: String,
}

impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
            html_output: String::new(),
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

    // Tokens mirroring the terminals in the grammar
    #[inline] fn is_hai(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#HAI") }
    #[inline] fn is_kbye(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#KBYE") }
    #[inline] fn is_obtw(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#OBTW") }
    #[inline] fn is_tldr(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#TLDR") }
    #[inline] fn is_maek(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#MAEK") }
    #[inline] fn is_oic(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#OIC") }
    #[inline] fn is_gimmeh(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#GIMMEH") }
    #[inline] fn is_mkay(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#MKAY") }
    #[inline] fn is_head(&self, s: &str) -> bool { s.eq_ignore_ascii_case("HEAD") }
    #[inline] fn is_title(&self, s: &str) -> bool { s.eq_ignore_ascii_case("TITLE") }
    #[inline] fn is_paragraf(&self, s: &str) -> bool { s.eq_ignore_ascii_case("PARAGRAF") }
    #[inline] fn is_bold(&self, s: &str) -> bool { s.eq_ignore_ascii_case("BOLD") }
    #[inline] fn is_list(&self, s: &str) -> bool { s.eq_ignore_ascii_case("LIST") }
    #[inline] fn is_item(&self, s: &str) -> bool { s.eq_ignore_ascii_case("ITEM") }
    #[inline] fn is_linx(&self, s: &str) -> bool { s.eq_ignore_ascii_case("LINX") }
    #[inline] fn is_newline(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#NEWLINE") }
    #[inline] fn is_ihaz(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#IHAZ") }
    #[inline] fn is_var(&self, s: &str) -> bool { s.eq_ignore_ascii_case("VAR") }
    
    
    fn hai(&mut self) {
        if self.is_hai(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#HAI' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn kbye(&mut self) {
        if self.is_kbye(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#KBYE' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn obtw(&mut self) {
        if self.is_obtw(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#OBTW' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn tldr(&mut self) {
        if self.is_tldr(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#TLDR' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn maek(&mut self) {
        if self.is_maek(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#MAEK' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn mkay(&mut self) {
        if self.is_mkay(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#MKAY' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn gimmeh(&mut self) {
        if self.is_gimmeh(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#GIMMEH' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn oic(&mut self) {
        if self.is_oic(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#OIC' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn head_name(&mut self) {
        if self.is_head(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'HEAD' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn title_name(&mut self) {
        if self.is_title(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'TITLE' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn paragraf_name(&mut self) {
        if self.is_paragraf(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'PARAGRAF' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn bold_name(&mut self) {
        if self.is_bold(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'BOLD' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn list_name(&mut self) {
        if self.is_list(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'LIST' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn item_name(&mut self) {
        if self.is_item(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'ITEM' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn linx_name(&mut self) {
        if self.is_linx(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'LINX' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn newline_token(&mut self) {
        if self.is_newline(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#NEWLINE' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn ihaz(&mut self) {
        if self.is_ihaz(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when '#IHAZ' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn var_keyword(&mut self) {
        if self.is_var(&self.current_tok) {
            let _ = self.next_token();
        } else {
            eprintln!("Syntax error: '{}' was found when 'VAR' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn varname(&mut self) {
        let _ = self.next_token();
    }

    fn varvalue(&mut self) {
        let _ = self.next_token();
    }

    fn address(&mut self) {
        let _ = self.next_token();
    }

    fn parse_lolcode(&mut self){
        self.hai();
        self.parse_comment();
        self.parse_head();
        self.parse_variable_define(); // optional
        self.parse_body();
        self.kbye();
    }

    fn parse_head(&mut self){
        if self.is_maek(&self.current_tok) {
            self.maek();
            self.head_name();
            self.parse_title();
            self.mkay();
        }
    }

    fn parse_title(&mut self){
        self.gimmeh();
        self.title_name();
        self.parse_text();
        self.oic();
    }

    fn parse_comment(&mut self){
        self.obtw();
        self.parse_text();
        self.tldr();
    }

    fn parse_body(&mut self){
        if self.is_obtw(&self.current_tok) {
            self.parse_comment();
        } else if self.is_newline(&self.current_tok) {
            self.parse_newline();
        } else if self.is_ihaz(&self.current_tok) {
            self.parse_variable_use();
        } else if self.is_maek(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_paragraf(&peek) {
                self.parse_paragraph();
            } else if self.is_list(&peek) {
                self.parse_list();
            } else {
                self.parse_text();
            }
        } else if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else {
                self.parse_text();
            }
        } else {
            self.parse_text();
        }
        self.parse_body();
    }


    fn parse_paragraph(&mut self){
        self.maek();
        self.paragraf_name();
        self.parse_variable_define(); // optional
        self.parse_inner_paragraph();
        self.mkay();
    }

    fn parse_inner_paragraph(&mut self){
        if !self.current_tok.is_empty() && !self.is_mkay(&self.current_tok) {
            self.parse_inner_text();
            self.parse_inner_paragraph();
        }        
    }

    fn parse_inner_text(&mut self){
        if self.is_ihaz(&self.current_tok) {
            self.parse_variable_use();
        } else if self.is_obtw(&self.current_tok) {
            self.parse_comment();
        } else if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else {
                self.parse_text();
            }
        } else if self.is_maek(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_list(&peek) {
                self.parse_list();
            } else {
                self.parse_text();
            }
        } else if self.is_newline(&self.current_tok) {
            self.parse_newline();
        } else {
            self.parse_text();
        }
    }

    fn parse_variable_define(&mut self){
        if self.is_var(&self.current_tok) {
            self.var_keyword();
            self.varname();
            self.varvalue();
        }
    }

    fn parse_variable_use(&mut self){
        self.ihaz();
        self.varname();
    }

    fn parse_bold(&mut self){
        self.gimmeh();
        self.bold_name();
        self.parse_text();
        self.oic();
    }

    fn parse_list(&mut self){
        self.gimmeh();
        self.bold_name();
        self.parse_text();
        self.oic();
    }

    fn parse_list_items(&mut self){
        if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_item(&peek) {
                self.gimmeh();
                self.item_name();
                self.parse_variable_define(); // optional
                self.parse_inner_list();
                self.oic();
                self.parse_list_items();
            }
        }
    }

    fn parse_inner_list(&mut self){
        if self.is_obtw(&self.current_tok) {
            self.parse_comment();
        } else if self.is_newline(&self.current_tok) {
            self.parse_newline();
        } else if self.is_ihaz(&self.current_tok) {
            self.parse_variable_use();
        } else if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else {
                self.parse_text();
            }
        } else if self.is_maek(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_list(&peek) {
                self.parse_list();
            } else {
                self.parse_text();
            }
        } else {
            self.parse_text();
        }
        self.parse_inner_list();
    }

    fn parse_link(&mut self){
        self.gimmeh();
        self.linx_name();
        self.address();
        self.oic();
    }

    fn parse_newline(&mut self){
        self.newline_token();
    }

    fn parse_text(&mut self){
        let _ = self.next_token();
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
        self.parse_lolcode();
        if !self.lexer.tokens.is_empty() || !self.current_tok.is_empty() {
            eprintln!("Syntax error: Additional tokens found after the sentence.");
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String { self.current_tok.clone() }
    fn set_current_token(&mut self, tok: String) { self.current_tok = tok; }
}

// -- Cross-platform Browser Function
fn open_in_browser(path: &str) -> io::Result<ExitStatus> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", path])
            .status()
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .status()
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    {
        Command::new("xdg-open")
            .arg(path)
            .status()
    }
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


    /*let html_file = "index.html";
    
    match open_in_browser(html_file) {
        Ok(status) if status.success() => println!("Successfully opened {} in the default browser.", html_file),
        Ok(status) => eprintln!("Browser command exited with an error status: {}", status),
        Err(e) => eprintln!("Failed to execute the system command: {}", e),
    }*/

}
