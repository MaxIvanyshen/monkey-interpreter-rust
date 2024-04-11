use token::{Token, TokenType};


pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: char, 
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut l = Lexer {
            input: input.to_string(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let tok = match self.ch {
            ';' => Token::new(TokenType::SEMICOLON, self.ch.to_string()),
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    return Token::new(TokenType::EQ, "==".to_string());
                }
                Token::new(TokenType::ASSIGN, self.ch.to_string())
            },
            '+' => Token::new(TokenType::PLUS, self.ch.to_string()),   
            '-' => Token::new(TokenType::MINUS, self.ch.to_string()),   
            '*' => Token::new(TokenType::ASTERISK, self.ch.to_string()),   
            '/' => Token::new(TokenType::SLASH, self.ch.to_string()),   
            '<' => Token::new(TokenType::LT, self.ch.to_string()),   
            '>' => Token::new(TokenType::RT, self.ch.to_string()),   
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    return Token::new(TokenType::NOT_EQ, "!=".to_string());
                }
                Token::new(TokenType::BANG, self.ch.to_string())
            },
            '(' => Token::new(TokenType::LPAREN, self.ch.to_string()),   
            ')' => Token::new(TokenType::RPAREN, self.ch.to_string()),   
            '{' => Token::new(TokenType::LBRACE, self.ch.to_string()),   
            '}' => Token::new(TokenType::RBRACE, self.ch.to_string()),   
            ',' => Token::new(TokenType::COMMA, self.ch.to_string()),   
            '%' => Token::new(TokenType::MODULO, self.ch.to_string()),
            '\0' => Token::new(TokenType::EOF, self.ch.to_string()),
            _ => {
                if self.ch.is_alphabetic() {
                    let mut tok = self.read_identifier();
                    if token::lookup_ident(&tok.literal).to_string() != TokenType::IDENT.to_string() {
                        tok.token_type = token::lookup_ident(&tok.literal);
                    }
                    tok
                } else if self.ch.is_digit(10) {
                    self.read_number()
                } else if self.ch == '"' {
                    self.read_string()  
                } else {
                    Token::new(TokenType::ILLEGAL, self.ch.to_string())
                }
            }
        };

        self.read_char();
        tok
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while self.ch.is_alphabetic() {
            ident.push(self.ch);
            self.read_char();
        }
        self.revert_char();

        Token::new(TokenType::IDENT, ident)
    }

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        while self.ch.is_digit(10) {
            number.push(self.ch);
            self.read_char();
        }
        self.revert_char();

        Token::new(TokenType::INT, number)
    }

    fn read_string(&mut self) -> Token {
        self.read_char();
        let mut str = String::new();
        while self.ch != '"' {
            str.push(self.ch);
            self.read_char();
        }

        Token::new(TokenType::STRING, str)
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input.chars().nth(self.read_position).unwrap()
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() || self.ch == '\n' {
            self.read_char();
        }
    }

    fn revert_char(&mut self) {
        self.read_position = self.position;
        self.position -= 1;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "
        let five = 5;
        let b = true;
        let b = false;
        let ten = 5 + 5;
        let zero = 5 - 5;
        let ten = 5 * 2;
        */=!;
        == != < > ;

        fn(x, y) {
            x + y;
        };

        if(5 < 10) {
            return true;
        } else {
            return false;
        }

        let s = \"hello world\";
                        ";
        let mut lexer = Lexer::new(input);

        let tests = vec![
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "five".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "b".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::TRUE, "true".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "b".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::FALSE, "false".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "ten".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::PLUS, "+".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "zero".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::MINUS, "-".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "ten".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::ASTERISK, "*".to_string()),
            Token::new(TokenType::INT, "2".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::ASTERISK, "*".to_string()),
            Token::new(TokenType::SLASH, "/".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::BANG, "!".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::EQ, "==".to_string()),
            Token::new(TokenType::NOT_EQ, "!=".to_string()),
            Token::new(TokenType::LT, "<".to_string()),
            Token::new(TokenType::RT, ">".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::FUNCTION, "fn".to_string()),
            Token::new(TokenType::LPAREN, "(".to_string()),
            Token::new(TokenType::IDENT, "x".to_string()),
            Token::new(TokenType::COMMA, ",".to_string()),
            Token::new(TokenType::IDENT, "y".to_string()),
            Token::new(TokenType::RPAREN, ")".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::IDENT, "x".to_string()),
            Token::new(TokenType::PLUS, "+".to_string()),
            Token::new(TokenType::IDENT, "y".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::IF, "if".to_string()),
            Token::new(TokenType::LPAREN, "(".to_string()),
            Token::new(TokenType::INT, "5".to_string()),
            Token::new(TokenType::LT, "<".to_string()),
            Token::new(TokenType::INT, "10".to_string()),
            Token::new(TokenType::RPAREN, ")".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::RETURN, "return".to_string()),
            Token::new(TokenType::TRUE, "true".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::ELSE, "else".to_string()),
            Token::new(TokenType::LBRACE, "{".to_string()),
            Token::new(TokenType::RETURN, "return".to_string()),
            Token::new(TokenType::FALSE, "false".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::RBRACE, "}".to_string()),
            Token::new(TokenType::LET, "let".to_string()),
            Token::new(TokenType::IDENT, "s".to_string()),
            Token::new(TokenType::ASSIGN, "=".to_string()),
            Token::new(TokenType::STRING, "hello world".to_string()),
            Token::new(TokenType::SEMICOLON, ";".to_string()),
            Token::new(TokenType::EOF, '\0'.to_string()),
        ];

        for tt in tests {
            let tok = lexer.next_token();
            assert_eq!(tok.token_type.to_string(), tt.token_type.to_string());
            assert_eq!(tok.literal, tt.literal);
        }
    }

}
