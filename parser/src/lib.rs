use std::rc::Rc;
use ast::InfixExpression;
use lexer::Lexer;
use token::{Token, TokenType};
use std::collections::HashMap;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    LOWEST = 1,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

type PrefixParseFn = fn(&mut Parser) -> Option<Rc<dyn ast::Expression>>;
type InfixParseFn = fn(&mut Parser, Rc<dyn ast::Expression>) -> Option<Rc<dyn ast::Expression>>;

pub struct Parser {
    lexer: Lexer,

    current_token: Rc<Token>,
    peek_token: Rc<Token>,

    errors: Vec<String>,

    prefix_parse_fns: HashMap<token::TokenType, PrefixParseFn>,
    infix_parse_fns: HashMap<token::TokenType, InfixParseFn>
}

impl Parser {

    pub fn new(mut lexer: Lexer) -> Self {
        let prefix_parse_fns = HashMap::new();
        let infix_parse_fns = HashMap::new();

        let mut p = Parser {
            current_token: Rc::new(lexer.next_token()),
            peek_token: Rc::new(lexer.next_token()),
            lexer,
            prefix_parse_fns,
            infix_parse_fns,
            errors: vec![],
        };

        p.register_prefix(TokenType::IDENT, Parser::parse_identifier);
        p.register_prefix(TokenType::INT, Parser::parse_integer_literal);
        p.register_prefix(TokenType::STRING, Parser::parse_string_literal);
        p.register_prefix(TokenType::TRUE, Parser::parse_boolean);
        p.register_prefix(TokenType::FALSE, Parser::parse_boolean);
        p.register_prefix(TokenType::BANG, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::MINUS, Parser::parse_prefix_expression);
        p.register_prefix(TokenType::LPAREN, Parser::parse_grouped_expression);
        p.register_prefix(TokenType::IF, Parser::parse_if_expression);
        p.register_prefix(TokenType::FUNCTION, Parser::parse_function_literal);

        p.register_infix(TokenType::PLUS, Parser::parse_infix_expression);
        p.register_infix(TokenType::MINUS, Parser::parse_infix_expression);
        p.register_infix(TokenType::SLASH, Parser::parse_infix_expression);
        p.register_infix(TokenType::ASTERISK, Parser::parse_infix_expression);
        p.register_infix(TokenType::LT, Parser::parse_infix_expression);
        p.register_infix(TokenType::RT, Parser::parse_infix_expression);
        p.register_infix(TokenType::EQ, Parser::parse_infix_expression);
        p.register_infix(TokenType::NOT_EQ, Parser::parse_infix_expression);
        p.register_infix(TokenType::LPAREN, Parser::parse_call_expression);
        p.register_infix(TokenType::MODULO, Parser::parse_infix_expression);
        p.register_infix(TokenType::STRING, Parser::parse_infix_expression);
        
        p
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = Rc::new(self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> ast::Program {
        let mut program = ast::Program {
            statements: vec![]
        };
    
        while self.current_token.token_type.to_string() != "EOF" {
            let stmt = self.parse_statement();
            if stmt.is_some() {
                program.statements.push(stmt.unwrap());
            }
            self.next_token();
        }

        program
    }
    
    fn parse_statement(&mut self) -> Option<Rc<dyn ast::Statement>> {
        match self.current_token.clone().token_type {
            TokenType::LET => self.parse_let_statement(),
            TokenType::RETURN => self.parse_return_statement(),
            TokenType::LBRACE => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Rc<dyn ast::Statement>> {
        let token = self.current_token.clone();
        let expression = self.parse_expression(Precedence::LOWEST);
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Some(Rc::new(ast::ExpressionStatement {
            token,
            expression,
        }))
    }

    fn parse_let_statement(&mut self) -> Option<Rc<dyn ast::Statement>> {
        let token = self.current_token.clone();
    
        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let name = Rc::new(ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.clone().literal.clone(),
        });

        if !self.expect_peek(TokenType::ASSIGN) {
            return None;
        }

        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST);

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Rc::new(ast::LetStatement {
            token,
            name,
            value,
        }))
    }

    fn parse_string_literal(&mut self) -> Option<Rc<dyn ast::Expression>> {
        Some(Rc::new(ast::StringLiteral {
            token: self.current_token.clone(), 
            value: self.current_token.literal.clone(),
        }))
    }

    fn parse_return_statement(&mut self) -> Option<Rc<dyn ast::Statement>> {
        let token = self.current_token.clone();
        self.next_token();
        let return_value = self.parse_expression(Precedence::LOWEST);

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Rc::new(ast::ReturnStatement {
            token,
            return_value,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<Rc<dyn ast::Statement>> {
        let token = self.current_token.clone();
        let mut statements = vec![];

        self.next_token();

        while !self.current_token_is(TokenType::RBRACE) && !self.current_token_is(TokenType::EOF) {
            let stmt = self.parse_statement();
            if stmt.is_some() {
                statements.push(stmt.unwrap());
            }
            self.next_token();
        }

        Some(Rc::new(ast::BlockStatement {
            token,
            statements,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Rc<dyn ast::Expression>> {
        let curr_token_type = self.current_token.token_type.clone();
        let prefix = self.prefix_parse_fns.get(&curr_token_type);
        if prefix.is_none() {
            self.no_prefix_parse_fn_error(curr_token_type);
            return None;
        }

        let mut left_exp = prefix.unwrap()(self);

        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < Parser::get_precedence(self.peek_token.clone().token_type) {
            let peek_token_type = self.peek_token.token_type.clone();
            let infix = self.infix_parse_fns.get(&peek_token_type);
            if infix.is_none() {
                return left_exp;
            }

            self.current_token = self.peek_token.clone();
            self.peek_token = Rc::new(self.lexer.next_token());

            left_exp = infix.unwrap()(self, left_exp.unwrap());
        }

        left_exp

    }

    fn parse_integer_literal(&mut self) -> Option<Rc<dyn ast::Expression>> {
        let value = self.current_token.literal.parse::<i64>();

        if value.is_err() {
            let msg = format!("could not parse {} as integer", self.current_token.literal);
            self.errors.push(msg);
            return None;
        }

        Some(Rc::new(ast::IntegerLiteral {
            token: self.current_token.clone(),
            value: value.unwrap(),
        }))
    }

    fn parse_identifier(&mut self) -> Option<Rc<dyn ast::Expression>> {
        Some(Rc::new(ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        }))
    }

    fn parse_boolean(&mut self) -> Option<Rc<dyn ast::Expression>> {
        Some(Rc::new(ast::Boolean {
            token: self.current_token.clone(),
            value: self.current_token_is(TokenType::TRUE),
        }))
    }

    fn parse_prefix_expression(&mut self) -> Option<Rc<dyn ast::Expression>> {
        let operator = &self.current_token.clone().literal;
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX).unwrap();
        Some(Rc::new(
            ast::PrefixExpression {
                token: self.current_token.clone(),
                operator: operator.to_string(),
                right,
            }
        ))
    }

    fn parse_infix_expression(&mut self, left: Rc<dyn ast::Expression>) -> Option<Rc<dyn ast::Expression>> {
        let operator = &self.current_token.clone().literal;
        let token = self.current_token.clone();
        
        let precedence = Parser::get_precedence(self.current_token.clone().token_type);
        self.next_token();
        let right = self.parse_expression(precedence).unwrap();

        Some(Rc::new(
            InfixExpression {
                token,
                left,
                operator: operator.to_string(),
                right
            }
        ))
    }

    fn parse_grouped_expression(&mut self) -> Option<Rc<dyn ast::Expression>> {
        self.next_token();
        let exp = self.parse_expression(Precedence::LOWEST);
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        exp
    }

    fn get_precedence(token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::EQ => Precedence::EQUALS,
            TokenType::NOT_EQ => Precedence::EQUALS,
            TokenType::LT => Precedence::LESSGREATER,
            TokenType::RT => Precedence::LESSGREATER,
            TokenType::PLUS => Precedence::SUM,
            TokenType::MINUS => Precedence::SUM,
            TokenType::SLASH => Precedence::PRODUCT,
            TokenType::ASTERISK => Precedence::PRODUCT,
            TokenType::LPAREN => Precedence::CALL,
            TokenType::MODULO => Precedence::PRODUCT,
            _ => Precedence::LOWEST,
        }
    }

    fn parse_if_expression(&mut self) -> Option<Rc<dyn ast::Expression>> {
        let token = self.current_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST).unwrap();

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let if_body = self.parse_block_statement();

        if if_body.as_ref().is_none() {
            return None;
        }

        let mut if_exp = ast::IfExpression {
            token,
            condition,
            consequence: if_body.unwrap(),
            alternative: None,
        };

        if self.peek_token_is(TokenType::ELSE) {
            self.next_token();
            if !self.expect_peek(TokenType::LBRACE) {
                return None;
            }
            let alternative = self.parse_block_statement();
            if alternative.is_none() {
                return None;
            }
            if_exp.alternative = alternative;
        }

        Some(Rc::new(if_exp))
    }

    fn parse_function_literal(&mut self) -> Option<Rc<dyn ast::Expression>> {
        let token = self.current_token.clone();

        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        let parameters = self.parse_function_parameters();

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let body = self.parse_block_statement();

        if body.as_ref().is_none() {
            return None;
        }

        Some(Rc::new(ast::FunctionLiteral {
            token,
            parameters,
            body: body.unwrap(),
        }))
    }

    fn parse_function_parameters(&mut self) -> Vec<Rc<ast::Identifier>> {
        let mut identifiers = vec![];

        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return identifiers;
        }

        self.next_token();

        let ident = Rc::new(ast::Identifier {
            token: self.current_token.clone(),
            value: self.current_token.literal.clone(),
        });

        identifiers.push(ident);

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            let ident = Rc::new(ast::Identifier {
                token: self.current_token.clone(),
                value: self.current_token.literal.clone(),
            });
            identifiers.push(ident);
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return vec![];
        }

        identifiers
    }

    fn parse_call_expression(&mut self, function: Rc<dyn ast::Expression>) -> Option<Rc<dyn ast::Expression>> {
        let token = self.current_token.clone();
        let arguments = self.parse_call_arguments();
        Some(Rc::new(ast::CallExpression {
            token,
            function,
            arguments,
        }))
    }

    fn parse_call_arguments(&mut self) -> Vec<Rc<dyn ast::Expression>> {
        let mut args = vec![];

        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return args;
        }

        self.next_token();
        let arg = self.parse_expression(Precedence::LOWEST).unwrap();
        args.push(arg);

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            let arg = self.parse_expression(Precedence::LOWEST).unwrap();
            args.push(arg);
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return vec![];
        }

        args
    }

    fn current_token_is(&self, token_type: TokenType) -> bool {
        self.current_token.token_type.to_string() == token_type.to_string()
    }

    fn peek_token_is(&self, token_type: TokenType) -> bool {
        self.peek_token.token_type.to_string() == token_type.to_string()
    }

    fn expect_peek(&mut self, token_type: TokenType) -> bool {
        if self.peek_token_is(token_type) {
            self.next_token();
            true
        } else {
            self.add_peak_error(token_type);
            false
        }
    }

    fn add_peak_error(&mut self, token_type: TokenType) {
        let msg = format!("expected next token to be {}, got {} instead", token_type, self.peek_token.token_type);
        self.errors.push(msg);
    }

    fn no_prefix_parse_fn_error(&mut self, token_type: TokenType) {
        let msg = format!("no prefix parse function for {} found", token_type);
        self.errors.push(msg);
    }

    fn register_prefix(&mut self, token_type: TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }

    fn register_infix(&mut self, token_type: TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }
}

#[cfg(test)]
mod tests {
    use ast::Node;

    use super::*;

    #[test]
    fn test_parsing_let_statement() {
        let lexer = Lexer::new("let x = 5;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::LetStatement = program.statements[0].as_any().downcast_ref::<ast::LetStatement>().unwrap();
        assert_eq!(stmt.token_literal(), "let");
    }

    #[test]
    fn test_parsing_integer_literal() {
        let lexer = Lexer::new("5;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let value: &ast::IntegerLiteral = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        assert_eq!(value.value, 5);
    }

    #[test]
    fn test_parsing_string_literal() {
        let lexer = Lexer::new("\"hello\";");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let value: &ast::StringLiteral = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::StringLiteral>().unwrap();
        assert_eq!(value.value, "hello");
    }

    #[test]
    fn test_string_concatenation_parsing() {
        let lexer = Lexer::new("\"hello\" + \"world\";");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let infix: &ast::InfixExpression = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::InfixExpression>().unwrap();
        let left: &ast::StringLiteral = infix.left.as_ref().as_any().downcast_ref::<ast::StringLiteral>().unwrap();
        let right: &ast::StringLiteral = infix.right.as_ref().as_any().downcast_ref::<ast::StringLiteral>().unwrap();
        assert_eq!(left.value, "hello");
        assert_eq!(infix.operator, "+");
        assert_eq!(right.value, "world");
    }

    #[test]
    fn test_parsing_return_statement() {
        let lexer = Lexer::new("return 5;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ReturnStatement = program.statements[0].as_any().downcast_ref::<ast::ReturnStatement>().unwrap();
        let value: &ast::IntegerLiteral = stmt.return_value.as_ref().unwrap().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        assert_eq!(value.value, 5);
    }

    #[test]
    fn test_parsing_boolean() {
        let lexer = Lexer::new("true; false;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 2);

        let true_exp_stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let tru: &ast::Boolean = true_exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::Boolean>().unwrap();
        assert_eq!(tru.value, true);

        let false_exp_stmt: &ast::ExpressionStatement = program.statements[1].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let falsE: &ast::Boolean = true_exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::Boolean>().unwrap();
        assert_eq!(falsE.value, true);
    }

    #[test]
    fn test_simple_infix_expression() {
        let lexer = Lexer::new("5 + 5;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let infix: &ast::InfixExpression = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::InfixExpression>().unwrap();
        let left: &ast::IntegerLiteral = infix.left.as_ref().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        let right: &ast::IntegerLiteral = infix.right.as_ref().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        assert_eq!(left.value, 5);
        assert_eq!(infix.operator, "+");
        assert_eq!(right.value, 5);
    }

    #[test]
    fn test_parsing_prefix_expression() {
        let lexer = Lexer::new("!5; -15;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 2);
        let mut stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let mut infix: &ast::PrefixExpression = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::PrefixExpression>().unwrap();
        let mut right: &ast::IntegerLiteral = infix.right.as_ref().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        assert_eq!(infix.operator, "!");
        assert_eq!(right.value, 5);

        stmt = program.statements[1].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        infix = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::PrefixExpression>().unwrap();
        right = infix.right.as_ref().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
        assert_eq!(infix.operator, "-");
        assert_eq!(right.value, 15);
    }

    #[test]
    fn test_operator_precedence() {
        let lexer = Lexer::new("5 * 2 - 3 / 3;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let infix: &ast::InfixExpression = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::InfixExpression>().unwrap();
        assert_eq!(infix.to_string(), "((5 * 2) - (3 / 3))");
    }

    #[test]
    fn test_parsing_block_statement() {
       let lexer = Lexer::new("{
           let x = 5;
           let y = 10;
           let foobar = 838383;
       }"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
       assert_eq!(program.statements.len(), 1);

       let stmt = program.statements[0].as_any().downcast_ref::<ast::BlockStatement>().unwrap();
       assert_eq!(stmt.statements.len(), 3);
       assert_eq!(stmt.to_string(), "{let x = 5;let y = 10;let foobar = 838383;}");
    }

    #[test]
    fn test_parsing_if_statement() {
       let lexer = Lexer::new("if(x < y) {
           let x = 5;
           let y = 10;
           let foobar = 838383;
       } else {x}"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
       assert_eq!(program.statements.len(), 1);
       let exp_stmt = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
       let exp = exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::IfExpression>().unwrap();
       assert_eq!(exp.token_literal().to_string(), "if");
       assert_eq!(exp.condition.to_string(), "(x < y)");
       assert_eq!(exp.alternative.is_some(), true);
       assert_eq!(exp.to_string(), "if(x < y) {let x = 5;let y = 10;let foobar = 838383;} else {x}");
    }

    #[test]
    fn test_parsing_functions() {
       let lexer = Lexer::new("fn (x, y) {if(x < y) {
           let x = 5;
           let y = 10;
           let foobar = 838383;
       } else {x}}"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
       assert_eq!(program.statements.len(), 1);
       let exp_stmt = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
       let exp = exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::FunctionLiteral>().unwrap();
       assert_eq!(exp.to_string(), "fn(x, y) {if(x < y) {let x = 5;let y = 10;let foobar = 838383;} else {x}}");
    }

    #[test]
    fn test_parsing_call_expresssions_0_args() {
       let lexer = Lexer::new("add();"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
       assert_eq!(program.statements.len(), 1);
       let exp_stmt = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
       let exp = exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::CallExpression>().unwrap();
       assert_eq!(exp.arguments.len(), 0);
       assert_eq!(exp.function.token_literal(), "add");
       assert_eq!(exp.to_string(), "add()");
    }

    #[test]
    fn test_parsing_call_expresssions_2_args() {
       let lexer = Lexer::new("add(x, y);"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
       assert_eq!(program.statements.len(), 1);
       let exp_stmt = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
       let exp = exp_stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::CallExpression>().unwrap();
       assert_eq!(exp.arguments.len(), 2);
       assert_eq!(exp.function.token_literal(), "add");
       assert_eq!(exp.to_string(), "add(x, y)");
    }

    #[test]
    fn test_parsing_mixed_expression() {
       let lexer = Lexer::new("-3 + !add(x, y) * 2"); 
       let mut parser = Parser::new(lexer);
       let program = parser.parse_program();
        assert_eq!(program.statements.len(), 1);
        let stmt: &ast::ExpressionStatement = program.statements[0].as_any().downcast_ref::<ast::ExpressionStatement>().unwrap();
        let infix: &ast::InfixExpression = stmt.expression.as_ref().unwrap().as_any().downcast_ref::<ast::InfixExpression>().unwrap();
        assert_eq!(infix.to_string(), "((-3) + ((!add(x, y)) * 2))");
    }

    #[test]
    fn test_catching_parsing_error() {
       let lexer = Lexer::new("let x;"); 
       let mut parser = Parser::new(lexer);
       let _program = parser.parse_program();
       assert_eq!(parser.errors().len(), 2);
    }
}
