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

/// -- Lexical Analyzer
/// A lexical analyzer that tokenizes the input into valid tokens based on whitespace 
/// and checks their validity against a set of start tags, end tags, tag names, and statements.
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

/// Implementation of the SimpleLexicalAnalyzer struct
/// Provides the methods for tokenizing input and checking token validity
impl SimpleLexicalAnalyzer {
    /// Creates a new instance of the SimpleLexicalAnalyzer
    /// Outputs a SimpleLexicalAnalyzer struct with the input source, position at 0, empty current_build, empty tokens, and predefined start_tags, end_tags, tag_names, and statements
    pub fn new (source: &str) -> Self {
        Self { // initalizes self with input from source, position at 0 (start of the input), empty current_build, empty tokens , predefined start_tags, end_tags, tag_names, and statements
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

    /// Tokenizes the input into valid tokens by iterating through each character in the input, 
    /// putting the characters together into tokens, and checking their validity
    pub fn tokenize(&mut self) { // tokenizes the input into valid tokens by iterating through each character in the input, putting the chars together into tokens, and checking their validity
        loop {
            let c = self.get_char();
            if c == '\0' {
                break;
            }
            if c.is_whitespace() { /// if the character is whitespace, check if the current_build is not empty (meaning a token has been built)
                if !self.current_build.is_empty() {
                    let token = std::mem::take(&mut self.current_build);
                    self.push_valid_token(token);
                }
            } else { /// else keep adding characters to the current_build to form a token
                self.add_char(c);
            }
        }
        if !self.current_build.is_empty() { /// if the current_build is not empty after the loop, push the last token to the tokens vector
            let token = std::mem::take(&mut self.current_build);
            self.push_valid_token(token);
        }
        self.tokens.reverse(); // reverse token for parsing
    }

    // push_valid_token helper
    fn push_valid_token(&mut self, token: String) { // checks if a token is valid. if it is, it is pushed to the tokens vector
        /// Checks if a token is valid. If it is, it is pushed to the tokens vector. If not, an error message is printed and the program exits
        if token.starts_with('#') {
            if !self.lookup(&token) {
                eprintln!("Lexical error: Unknown annotation '{}'", token);
                std::process::exit(1);
            }
        }
        self.tokens.push(token);
    }

}

/// Implementation of the LexicalAnalyzer trait for the SimpleLexicalAnalyzer struct
/// Handles getting the next char, adding the char to the current token, and checking token validity
impl LexicalAnalyzer for SimpleLexicalAnalyzer {
    fn get_char(&mut self) -> char { /// outputs next char 
        if self.position < self.input.len() { /// check if the current position is less than the length of the input
            let c = self.input[self.position]; // get the character at current position
            self.position += 1; /// increment position
            c // return the character
        } else { // else, return null
            '\0'
        }
    }

    fn add_char(&mut self, c: char) { /// adds a character to the current token from the input stream
        self.current_build.push(c);
    }

    fn lookup(&self, s: &str) -> bool { /// checks for valid tokens
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

/// A struct representing the Lolcode compiler, which handles the parsing and compilation of Lolcode source files
pub struct LolcodeCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    pub html_output: String, // HTML output stored as a String, will be appended to as the compiler parses the input
    vars: Vec<HashMap<String, String>>, // stack to handle variable scopes, formatted as <variable_name, variable_value>
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

    /// Helper functions to manage scope (which is handled by a stack of hashmaps)
    fn push_scope(&mut self) {
        self.vars.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.vars.len() > 1 {
            self.vars.pop();
        }
    }

    /// Starts parsing by checking the first token, setting it as the current token, and checking its validity
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

    /// Tokens mirroring the terminals in the grammar
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
    
    /// -- terminal functions
    /// all terminal functions check if the current token matches the expected token
    /// If yes, they consume it and move to the next token (and append the corresponding HTML tag if applicable)
    /// If not, they print an error message and exit
    
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

    /// Helper functions to parse variable names, values, and addresses (used in links)
    fn varname(&mut self) {
        /// retrieves the next token and stores it as the variable name
        let _ = self.next_token();
    }

    fn varvalue(&mut self) {
        /// retrieves the next token and stores it as the variable value
        let _ = self.next_token();
    }

    fn address(&mut self) {
        /// retrieves the next token and stores it as the address for links
        let _ = self.next_token();
    }
}
    
impl SyntaxAnalyzer for LolcodeCompiler {
    /// -- non-terminal functions
  
    /// Parses the entire lolcode input, following the format: HAI <comments> <head> <optional-variable-define> <body> KBYE
    fn parse_lolcode(&mut self){
        self.hai(); // #HAI
        while self.is_obtw(&self.current_tok) {
            self.parse_comment(); // comments (optional)
        }
        self.parse_head(); // head
        self.parse_variable_define(); // optional variable
        self.parse_body(); // body
        self.kbye(); // #KBYE
    }

    /// Parses the head section of the lolcode input, following the format: MAEK HEAD <title> MKAY
    /// appends the corresponding <head> and </head> tags to the html_output
    fn parse_head(&mut self){
        if self.is_maek(&self.current_tok) && self.is_head(&self.lexer.tokens.last().cloned().unwrap_or_default()) { /// check specifically for #MAEK HEAD
            self.maek(); // #MAEK
            self.head_name(); // HEAD
            self.parse_title(); // parse title inside head
            self.mkay(); // #MKAY
            self.html_output.push_str("</head>")
        }
    }

    /// Parses the title section of the lolcode input, following the format: GIMMEH TITLE <text> OIC
    /// Appends the corresponding <title> and </title> tags to the html_output
    fn parse_title(&mut self){
        self.gimmeh(); // #GIMMEH
        self.title_name(); // TITLE
        self.parse_text(); // parse text inside title
        self.oic(); // #OIC
        self.html_output.push_str("</title>"); // append </title> tag to html_output
    }

    /// Parses the comment section of the lolcode input, following the format: OBTW <text> TLDR
    fn parse_comment(&mut self){
        self.obtw(); // #OBTW
        self.html_output.push_str("<!-- "); // append comment start tag to html_output
        self.parse_text();
        self.tldr(); // #TLDR
        self.html_output.push_str(" -->"); // append comment end tag to html_output
    }

    /// Parses the body section of the lolcode input
    /// which can include paragraphs, lists, comments, newlines, variable definitions, variable uses, bold text, links, and regular text
    fn parse_body(&mut self){
        /// append <body> tag to html_output
        self.html_output.push_str("<body>"); 
        while !self.current_tok.is_empty() && !self.is_kbye(&self.current_tok) { // while there are tokens and the current token is not #KBYE, parse the body
            /// if the current token is #MAEK: paragraph or list
            if self.is_maek(&self.current_tok) { 
                let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
                if self.is_paragraf(&peek) {
                    self.parse_paragraph();
                } else if self.is_list(&peek) {
                    self.parse_list();
                } else {
                    self.parse_text(); // outputs error from parse_text
                }
            } else if self.is_obtw(&self.current_tok) {/// #OBTW -> comment
                self.parse_comment();
            } else if self.is_newline(&self.current_tok) {
                self.parse_newline();
            } else if self.is_ihaz(&self.current_tok) { /// if the current token is #IHAZ, parse variable
                let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
                if self.is_var(&peek) {
                    self.parse_variable_use();
                } else {
                    self.parse_variable_define();
                }    
            } else if self.is_gimmeh(&self.current_tok) { /// else, if the current token is #GIMMEH: bold, link, variable use, or text
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
        self.html_output.push_str("</body>");
    }


    /// Parses a paragraph section of the lolcode input, following the format: #MAEK PARAGRAF <optional-variable-define> <inner-paragraph> #MKAY
    fn parse_paragraph(&mut self){
        self.push_scope(); /// create scope for variables in paragraph
        self.maek();
        self.paragraf_name(); // PARAGRAF and appends <p> 

        if self.is_ihaz(&self.current_tok) { /// optional variable define
            self.parse_variable_define();
        }

        self.parse_inner_paragraph();
        self.mkay();
        self.html_output.push_str("</p>");
        self.pop_scope(); // remove paragraph scope
    }

    /// Parses the inner paragraph section of the lolcode input
    /// Calls parse_inner_text repeatedly until the end tag is found or there are no more tokens
    fn parse_inner_paragraph(&mut self){
        // parse inner paragraph until end tag is found/no more tokens
        if !self.current_tok.is_empty() && !self.is_mkay(&self.current_tok) {
            self.parse_inner_text();
            self.parse_inner_paragraph();
        }
    }

    /// Parses the inner text section of the lolcode input,
    /// which can include comments, newlines, bold text, links, variable uses, and regular text
    fn parse_inner_text(&mut self){
        if self.is_ihaz(&self.current_tok) { /// if the current token is #IHAZ, parse variable
            self.parse_variable_use();
        } else if self.is_obtw(&self.current_tok) { /// if the current token is #OBTW, parse comment
            self.parse_comment();
        } else if self.is_gimmeh(&self.current_tok) { /// else, if the current token is #GIMMEH: bold, link, variable use, or text
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default(); /// peek at the next token to determine what to parse
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else if self.is_var(&peek) {
                self.parse_variable_use(); 
            } else {
                self.parse_text();
            }
        } else if self.is_maek(&self.current_tok) { /// else, if the current token is '#MAEK': list or text
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default(); // peek at the next token to determine what to parse
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

    /// Parses a variable definition section of the lolcode input, following the format: #IHAZ <variable-name> #ITIZ <variable-value>
    /// Inserts a valid variable name and value into the current scope in the vars stack
    fn parse_variable_define(&mut self){
        if self.is_ihaz(&self.current_tok) { /// check if the current token is #IHAZ
            self.ihaz();
            let name = self.current_tok.clone(); // setting retrieved name token to name
            self.varname(); // parse the variable name
            if self.is_itiz(&self.current_tok) { /// check if the current token is #ITIZ
                let _ = self.next_token();
            } else { /// else, print error
                eprintln!("Syntax error: '{}' was found when '#ITIZ' was expected.", self.current_tok);
                std::process::exit(1);
            }
            let val = self.current_tok.clone(); // setting retrieved value token to var
            self.varvalue(); // parse the variable value
            self.mkay();
            
            // let current scope be the last scope in the vars stack (scopes), and insert the variable name and value into it
            let current_scope = self.vars.last_mut().unwrap();
            current_scope.insert(name, val); // insert the variable name and value into the scope
        }
    }

    /// Parses a variable use section of the lolcode input, following the format: #GIMMEH VAR <variable-name> #OIC
    /// Searches for a valid variable in the scopes stack (vars) and retrieves its value if found, appending it to the html_output
    fn parse_variable_use(&mut self){
        self.gimmeh(); // #GIMMEH
        self.var_keyword(); // VAR
        let var_name = self.current_tok.clone(); // retrieve the variable name token and store it in var_name
        self.varname(); // parse the variable name
        self.oic(); // #OIC

        /// search for the variable in the scopes stack (vars) and retrieve its value if found
        let val = self.vars.iter().rev()
            .find_map(|scope| scope.get(&var_name))
            .unwrap_or_else(|| { // if the variable is not found in any scope, print an error message and exit
                eprintln!("Error: variable '{}' is used but was never defined.", var_name);
                std::process::exit(1);
            });
        self.html_output.push_str(val); // append the variable value to the html_output
    }

    /// Parses a bold text section of the lolcode input, following the format: #GIMMEH BOLD <text> #OIC
    fn parse_bold(&mut self){
        self.gimmeh(); // #GIMMEH
        self.bold_name(); // BOLD
        self.parse_text(); // parse text to be bold
        self.oic(); // #OIC
        self.html_output.push_str("</b>"); // append </b> tag to html_output
    }

    /// Parses a list section of the lolcode input, following the format: #MAEK LIST <list-items> #MKAY
    /// List items are parsed recursively by the parse_list_items function, which handles each item in the list
    fn parse_list(&mut self){
        self.maek(); // #MAEK
        self.list_name(); // LIST
        if !self.is_gimmeh(&self.current_tok) { // requires a list item, so an error is thrown if not (#GIMMEH starts list items)
            eprintln!("Syntax error: '{}' was found when '#GIMMEH' was expected.", self.current_tok);
            std::process::exit(1);
        }
        self.parse_list_items(); // first item, then any further items
        self.mkay(); // #MKAY
        self.html_output.push_str("</ul>"); // append </ul> tag to html_output
    }

    /// Parses the list items section of the lolcode input, following the format: #GIMMEH ITEM <optional-variable-define> <inner-list> #OIC
    /// Contents of the list item are parsed recursively by the parse_inner_list function
    fn parse_list_items(&mut self){
        if self.is_gimmeh(&self.current_tok) {
            self.push_scope(); // scope for variables defined in this list item

            self.gimmeh(); // #GIMMEH
            self.item_name(); // ITEM
            self.parse_variable_define(); // optional variable
            self.parse_inner_list(); // get the content of the list item
            self.oic(); // #OIC
            self.html_output.push_str("</li>"); // append </li> tag to html_output

            self.pop_scope(); // remove list item scope

            self.parse_list_items(); // recursive call to parse the next list item
        }
    }

    /// function to parse the content of a list item, which can include comments, newlines, bold text, links, variable uses, and regular text
    fn parse_inner_list(&mut self){
        if self.is_oic(&self.current_tok) { // if #OIC is found (end of list item), return
            return;
        }

        if self.is_obtw(&self.current_tok) {
            self.parse_comment();
        } else if self.is_newline(&self.current_tok) {
            self.parse_newline();
        } else if self.is_gimmeh(&self.current_tok) {
            let peek = self.lexer.tokens.last().cloned().unwrap_or_default();
            if self.is_bold(&peek) {
                self.parse_bold();
            } else if self.is_linx(&peek) {
                self.parse_link();
            } else if self.is_var(&peek) {
                self.parse_variable_use();
            } else {
                self.parse_text(); // not valid list-content, caught by parse_text
            }
        } else {
            self.parse_text();
        }

        // recursive parse
        self.parse_inner_list();
    }

    /// Parses a link section of the lolcode input, following the format: #GIMMEH LINX <url> #OIC
    /// Address is parsed and the link is formatted in HTML format, appending it to the html_output
    fn parse_link(&mut self){
        self.gimmeh(); // #GIMMEH
        self.linx_name(); // LINX

        let url = self.current_tok.clone(); // retrieve the URL token and store it in url
        self.address(); // parse the URL address
        
        self.oic(); // #OIC
        let link_html = format!("<a href=\"{}\">{}</a>", url, url); // format in HTML link format
        self.html_output.push_str(&link_html);
    }

    /// Parses a newline section of the lolcode input
    fn parse_newline(&mut self){
        self.newline_token();
    }

    /// Parses regular text content, appending it to the html_output
    /// Checks for misplaced start/end tags and tag names, throwing a syntax error if found
    /// Loops until a special token (#) is found, appending the text to the html_output
    fn parse_text(&mut self){
        if self.current_tok.is_empty()
            || self.current_tok.starts_with('#') // check if the current token is a misplaced start/end tag
            || self.lexer.tag_names.iter().any(|t| t.eq_ignore_ascii_case(&self.current_tok))
        {
            eprintln!("Syntax error: '{}' was found when TEXT was expected.", self.current_tok);
            std::process::exit(1);
        }

        loop { // parse text until a special token (#) is found
            let val = self.current_tok.clone(); // store the current token in val
            self.html_output.push_str(&format!("{} ", val)); // append the text from val to the html_output
            let _ = self.next_token(); // get the next token

            if self.current_tok.is_empty() // check if the current token is empty
                || self.current_tok.starts_with('#') // check if the current token is a misplaced start/end tag
                || self.lexer.tag_names.iter().any(|t| t.eq_ignore_ascii_case(&self.current_tok)) // check if the current token is a tag name
            {
                break;
            }
        }
    }
}


/// Implementation of the Compiler trait for the LolcodeCompiler struct
/// Handles the compilation process, including tokenization, parsing, and error handling
impl Compiler for LolcodeCompiler {
    fn compile(&mut self, source: &str) {
        self.lexer = SimpleLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.start();
    }

    /// helper function to get next token from lexer
    fn next_token(&mut self) -> String {
        let candidate = self.lexer.tokens.pop().unwrap_or_default();
        if self.lexer.lookup(&candidate) { // check if candidate token is a valid token
            self.current_tok = candidate.clone(); // if it is, set candidate to current token and and return
            candidate
        } else if self.lexer.tokens.is_empty() { // if the candidate token is not valid and there are no more tokens, clear the current token and return an empty string
            self.current_tok.clear();
            String::new()
        } else {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate); // else, error
            std::process::exit(1);
        }
    }

    /// Parses the tokens generated by the lexer, checking for any remaining tokens after parsing to detect syntax errors
    fn parse(&mut self) {
        self.parse_lolcode(); // parse input gathered from parse_lolcode
        if !self.lexer.tokens.is_empty() || !self.current_tok.is_empty() { // checks if there are any remaining tokens after parsing (syntax error)
            eprintln!("Syntax error: Additional tokens found after the sentence.");
            std::process::exit(1);
        }
    }

    /// Returns the current token being processed
    fn current_token(&self) -> String { self.current_tok.clone() }
    /// Sets the current token being processed
    fn set_current_token(&mut self, tok: String) { self.current_tok = tok; }
}

/// -- Cross-platform Browser Function
/// Opens the given path in the default web browser, depending on the operating system
fn open_in_browser(path: &str) -> io::Result<ExitStatus> {
    #[cfg(target_os = "windows")] // if the target OS is windows, use cmd to open the file in the default browser
    {
        Command::new("cmd")
            .args(["/c", "start", "", path])
            .status()
    }

    #[cfg(target_os = "macos")] // target OS is macOS
    {
        Command::new("open")
            .arg(path)
            .status()
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))] // other OS
    {
        Command::new("xdg-open")
            .arg(path)
            .status()
    }
}

/// -- Main
/// The main function collects command line arguments, checks for a valid input file, reads the file content, compiles it 
/// using the LolcodeCompiler, and writes the output to an HTML file
/// It opens the generated HTML file in the default web browser
fn main() {
    /// Collect command line arguments
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

    /// create a new compiler instance and compile the input sentence
    let mut compiler = LolcodeCompiler::new();
    compiler.compile(&sentence); // &sentence is a reference to the sentence string, which is read from the input file
    compiler.parse(); // parses the tokens generated

    /// create a Path object from the input filename
    let path = Path::new(filename); 
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output"); // get the file stem (file name)
    let html_filename = format!("{}.html", stem); // create html_filename by combining the file stem (name) and .html file type

    fs::write(&html_filename, &compiler.html_output).unwrap_or_else(|err| { 
        eprintln!("Error writing HTML file '{}': {}", html_filename, err);
        std::process::exit(1);
    });

    /// get the path of the html file
    let absolute_path = fs::canonicalize(&html_filename) 
        .unwrap_or_else(|_| path.to_path_buf());
    let path_str = absolute_path.to_str().unwrap_or(&html_filename); // convert the path to a string

    match open_in_browser(path_str) { // open the html file path in the default browser
        Ok(status) if status.success() => println!("Successfully opened {} in the default browser.", html_filename),
        Ok(status) => eprintln!("Browser command exited with an error status: {}", status),
        Err(e) => eprintln!("Failed to execute the system command: {}", e),
    }

}
