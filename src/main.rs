use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
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

// -- Lexical Analyzer

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
                    let token = std::mem::take(&mut self.current_build);
                    self.push_valid_token(token);
                }
            } else {
                self.add_char(c);
            }
        }
        if !self.current_build.is_empty() {
            let token = std::mem::take(&mut self.current_build);
            self.push_valid_token(token);
        }
        self.tokens.reverse();
    }

    fn push_valid_token(&mut self, token: String) {
        if token.starts_with('#') {
            if !self.lookup(&token) {
                eprintln!("Lexical error: Unknown annotation '{}'", token);
                std::process::exit(1);
            }
        }
        self.tokens.push(token);
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
        if s.starts_with('#') {
            matches!(
                s.to_uppercase().as_str(),
                "#HAI" | "#KBYE" | "#OBTW" | "#TLDR" | "#MAEK" | "#OIC" | 
                "#GIMMEH" | "#MKAY" | "HEAD" | "TITLE" | "PARAGRAF" | 
                "BOLD" | "LIST" | "ITEM" | "#NEWLINE" | "LINX" | 
                "#IHAZ" | "#ITIZ" | "VAR"
            )
        } else {
            !s.is_empty()
        }
    }

}

// -- Syntax Analyzer

pub struct LolcodeCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    pub html_output: String,
    vars: Vec<HashMap<String, String>>,
}

impl LolcodeCompiler {
    pub fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
            html_output: String::new(),
            vars: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.vars.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.vars.len() > 1 {
            self.vars.pop();
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
    #[inline] fn is_itiz(&self, s: &str) -> bool { s.eq_ignore_ascii_case("#ITIZ") }
    #[inline] fn is_var(&self, s: &str) -> bool { s.eq_ignore_ascii_case("VAR") }
    
    // -- terminal functions
    
    fn hai(&mut self) {
        if self.is_hai(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<!DOCTYPE html>\n<html>\n");
        } else {
            eprintln!("Syntax error: '{}' was found when '#HAI' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn kbye(&mut self) {
        if self.is_kbye(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("</html>\n");
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
            self.html_output.push_str("<head>");
        } else {
            eprintln!("Syntax error: '{}' was found when 'HEAD' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn title_name(&mut self) {
        if self.is_title(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<title>");
        } else {
            eprintln!("Syntax error: '{}' was found when 'TITLE' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn paragraf_name(&mut self) {
        if self.is_paragraf(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<p>");
        } else {
            eprintln!("Syntax error: '{}' was found when 'PARAGRAF' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn bold_name(&mut self) {
        if self.is_bold(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<b>");
        } else {
            eprintln!("Syntax error: '{}' was found when 'BOLD' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn list_name(&mut self) {
        if self.is_list(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<ul>");
        } else {
            eprintln!("Syntax error: '{}' was found when 'LIST' was expected.", self.current_tok);
            std::process::exit(1);
        }
    }

    fn item_name(&mut self) {
        if self.is_item(&self.current_tok) {
            let _ = self.next_token();
            self.html_output.push_str("<li>");
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
            self.html_output.push_str("<br>");
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

    // -- non-terminal functions

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
            self.html_output.push_str("</head>")
        }
    }

    fn parse_title(&mut self){
        self.gimmeh();
        self.title_name();
        self.parse_text();
        self.oic();
        self.html_output.push_str("</title>");
    }

    fn parse_comment(&mut self){
        self.obtw();
        self.html_output.push_str("<!-- ");
        self.parse_text();
        self.tldr();
        self.html_output.push_str(" -->");
    }

    fn parse_body(&mut self){
        self.html_output.push_str("<body>");
        // parse inner body until end tag is found/no more tokens
        while !self.lexer.tokens.is_empty() && !self.is_kbye(&self.current_tok) {
            self.parse_inner_body();
        }
        self.html_output.push_str("</body>");
    }

    fn parse_inner_body(&mut self){
        self.push_scope();
        if self.is_maek(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_paragraf(&peek) {
                self.parse_paragraph();
            } else if self.is_list(&peek) {
                self.parse_list();
            } else {
                self.parse_text();
            }
        } else if self.is_obtw(&self.current_tok) {
            self.parse_comment();
        } else if self.is_newline(&self.current_tok) {
            self.parse_newline();
        } else if self.is_ihaz(&self.current_tok) {
            self.parse_variable_use();
        } else if self.is_ihaz(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_var(&peek) {
                self.parse_variable_use();
            } else {
                self.parse_variable_define();
            }    
        } else if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else if self.is_var(&peek) {
                self.parse_variable_use();
            } else {
                self.parse_text();
            }
        } else {
            self.parse_text();
        }
    }


    fn parse_paragraph(&mut self){
        self.push_scope(); // create scope for variables in paragraph
        self.maek();
        self.paragraf_name();
        self.html_output.push_str("<p>");

        if self.is_ihaz(&self.current_tok) { // optional variable define
            self.parse_variable_define();
        }

        self.parse_inner_paragraph();
        self.mkay();
        self.html_output.push_str("</p>");
        self.pop_scope(); // remove paragraph scope
    }

    fn parse_inner_paragraph(&mut self){
        while !self.current_tok.is_empty()
            && !self.is_mkay(&self.current_tok)
        {
            self.parse_inner_text();
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
            } else if self.is_var(&peek) {
                self.parse_variable_use(); 
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
        if self.is_ihaz(&self.current_tok) {
            self.ihaz();
            let name = self.current_tok.clone(); // setting retrieved name token to name
            self.varname();
            if self.is_itiz(&self.current_tok) {
                let _ = self.next_token();
            } else {
                eprintln!("Syntax error: '{}' was found when '#ITIZ' was expected.", self.current_tok);
                std::process::exit(1);
            }
            let val = self.current_tok.clone(); // setting retrieved value token to var
            self.varvalue();
            self.mkay();
            
            // let current scope be the last scope in the vars stack (scopes), and insert the variable name and value into it
            let current_scope = self.vars.last_mut().unwrap();
            current_scope.insert(name, val); // insert the variable name and value into the scope
        }
    }

    fn parse_variable_use(&mut self){
        self.gimmeh();
        self.var_keyword();
        let var_name = self.current_tok.clone();
        self.varname();
        self.oic();

        let mut val = var_name.clone();
        // check for variable in current scope
        for scope in self.vars.iter().rev() {
            if let Some(v) = scope.get(&var_name) { // if variable is found in the current scope, use its value
                val = v.clone();
                break;
            }
        }
        self.html_output.push_str(&format!("{}", val));
    }

    fn parse_bold(&mut self){
        self.gimmeh();
        self.bold_name();
        self.parse_text();
        self.oic();
        self.html_output.push_str("</b>");
    }

fn parse_list(&mut self){
        self.maek();
        self.list_name();
        self.parse_list_items();
        self.mkay();
        self.html_output.push_str("</ul>");
    }

    fn parse_list_items(&mut self){
        while self.is_gimmeh(&self.current_tok) {
            self.push_scope(); // create scope for variables in list item

            self.gimmeh();
            self.item_name();

            if self.is_ihaz(&self.current_tok) {
                self.parse_variable_define();
            }
            
            // Parse everything inside the list item iteratively
            self.parse_inner_list(); 
        
            self.oic();
            self.html_output.push_str("</li>");

            self.pop_scope(); // remove list item scope 
        }
    }

    fn parse_inner_list(&mut self){
        let mut item_depth = 0;

        while !self.current_tok.is_empty() {
            // If we hit an OIC at depth 0, it belongs to our list item—stop here!
            if self.is_oic(&self.current_tok) && item_depth == 0 {
                break;
            }

            // Track entering a nested list item
            if self.is_gimmeh(&self.current_tok) {
                // Peek ahead to see if it's an item start
                let peek = self.lexer.tokens.first().cloned().unwrap_or_default();
                if self.is_item(&peek) {
                    item_depth += 1;
                }
            } 
            // Track exiting a nested list item
            else if self.is_oic(&self.current_tok) {
                item_depth -= 1;
            }

            // Normal token processing inside the inner list
            if self.is_obtw(&self.current_tok) {
                self.parse_comment();
            } else if self.is_newline(&self.current_tok) {
                self.parse_newline();
            } else if self.is_ihaz(&self.current_tok) {
                self.parse_variable_use();
            } else if self.is_gimmeh(&self.current_tok) {
                self.gimmeh();
                let peek = self.lexer.tokens.first().cloned().unwrap_or_default();
                if self.is_bold(&peek) {
                    self.parse_bold();
                } else if self.is_linx(&peek) {
                    self.parse_link();
                } else if self.is_var(&peek) {
                    self.parse_variable_use();
                } else {
                    self.parse_text();
                }
            } else if self.is_maek(&self.current_tok) {
                let peek = self.lexer.tokens.first().cloned().unwrap_or_default();
                if self.is_list(&peek) {
                    self.parse_list();
                } else {
                    self.parse_text();
                }
            } else {
                self.parse_text();
            }
        }
    }

    fn parse_link(&mut self){
        self.gimmeh();
        self.linx_name();

        let url = self.current_tok.clone(); 
        self.address();
        
        self.oic();
        let link_html = format!("<a href=\"{}\">{}</a>", url, url);
        self.html_output.push_str(&link_html);
    }

    fn parse_newline(&mut self){
        self.newline_token();
    }

    fn parse_text(&mut self){
        if self.current_tok.is_empty()
            || self.current_tok.starts_with('#')
            || self.lexer.tag_names.iter().any(|t| t.eq_ignore_ascii_case(&self.current_tok))
        {
            eprintln!("Syntax error: '{}' was found when TEXT was expected.", self.current_tok);
            std::process::exit(1);
        }

        loop {
            let val = self.current_tok.clone();
            self.html_output.push_str(&format!("{} ", val));
            let _ = self.next_token();

            if self.current_tok.is_empty()
                || self.current_tok.starts_with('#')
                || self.lexer.tag_names.iter().any(|t| t.eq_ignore_ascii_case(&self.current_tok))
            {
                break;
            }
        }
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
    // collect command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { // check if input file is provided (args.len < 2 because args[0] is the program name and args[1] is the input file)
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1]; // set filename to args[1] (input file)

    if !filename.ends_with(".lol") { // check if input file has .lol extension
        eprintln!("Error: Invalid file extension, only .lol files are accepted.");
        std::process::exit(1);
    }

    let sentence = fs::read_to_string(filename).unwrap_or_else(|err| { // check if input file can be read/exists
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    // create a new compiler instance and compile the input sentence
    let mut compiler = LolcodeCompiler::new();
    compiler.compile(&sentence); // &sentence is a reference to the sentence string, which is read from the input file
    compiler.parse(); // parses the tokens generated

    // create a Path object from the input filename
    let path = Path::new(filename); 
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output"); // get the file stem (file name)
    let html_filename = format!("{}.html", stem); // create html_filename by combining the file stem (name) and .html file type

    fs::write(&html_filename, &compiler.html_output).unwrap_or_else(|err| { 
        eprintln!("Error writing HTML file '{}': {}", html_filename, err);
        std::process::exit(1);
    });

    // get the path of the html file
    let absolute_path = fs::canonicalize(&html_filename) 
        .unwrap_or_else(|_| path.to_path_buf());
    let path_str = absolute_path.to_str().unwrap_or(&html_filename); // convert the path to a string

    match open_in_browser(path_str) { // open the html file path in the default browser
        Ok(status) if status.success() => println!("Successfully opened {} in the default browser.", html_filename),
        Ok(status) => eprintln!("Browser command exited with an error status: {}", status),
        Err(e) => eprintln!("Failed to execute the system command: {}", e),
    }

}
