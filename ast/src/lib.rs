use token::Token;
use std::{fmt::Debug, rc::Rc};

#[derive(Debug)]
pub enum NodeType {
    PROGRAM,
    LET_STATEMENT,
    RETURN_STATEMENT,
    EXPRESSION_STATEMENT,
    INTEGER_LITERAL,
    STRING_LITERAL,
    PREFIX_EXPRESSION,
    INFIX_EXPRESSION,
    BOOLEAN,
    IF_EXPRESSION,
    BLOCK_STATEMENT,
    FUNCTION_LITERAL,
    CALL_EXPRESSION,
    IDENTIFIER,
}

pub trait Node {
    fn node_type(&self) -> NodeType;
    fn token_literal(&self) -> String;
    fn to_string(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Statement: Node + Debug {
    fn statement_node(&self);
}

pub trait Expression: Node + Debug {
    fn expression_node(&self);
}

pub struct Program {
    pub statements: Vec<Rc<dyn Statement>>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            String::from("")
        }
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        for s in &self.statements {
            out.push_str(&s.to_string());
        }
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::PROGRAM
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct Identifier {
    pub token: Rc<Token>,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.value.clone()
    }

    fn node_type(&self) -> NodeType {
        NodeType::IDENTIFIER
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub token: Rc<Token>,
    pub expression: Option<Rc<dyn Expression>>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        if let Some(expr) = &self.expression {
            expr.to_string()
        } else {
            String::new()
        }
    }

    fn node_type(&self) -> NodeType {
        NodeType::EXPRESSION_STATEMENT
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Statement for ExpressionStatement {
    fn statement_node(&self) {}
}

#[derive(Debug)]
pub struct LetStatement {
    pub token: Rc<Token>,
    pub name: Rc<Identifier>,
    pub value: Option<Rc<dyn Expression>>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.token_literal());
        out.push_str(" ");
        out.push_str(&self.name.to_string());
        out.push_str(" = ");
        if let Some(expr) = &self.value {
            out.push_str(&expr.to_string());
        }
        out.push_str(";");
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::LET_STATEMENT
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
}

#[derive(Debug)]
pub struct StringLiteral {
    pub token: Rc<Token>,
    pub value: String,
}

impl Node for StringLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("\"");
        out.push_str(&self.token.literal);
        out.push_str("\"");
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::STRING_LITERAL
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for StringLiteral {
    
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct IntegerLiteral {
    pub token: Rc<Token>,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.token.literal.clone()
    }

    fn node_type(&self) -> NodeType {
        NodeType::INTEGER_LITERAL
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for IntegerLiteral {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct PrefixExpression {
    pub token: Rc<Token>,
    pub operator: String,
    pub right: Rc<dyn Expression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("(");
        out.push_str(&self.operator);
        out.push_str(&self.right.to_string());
        out.push_str(")");
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::PREFIX_EXPRESSION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for PrefixExpression {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub token: Rc<Token>,
    pub return_value: Option<Rc<dyn Expression>>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.token_literal());
        out.push_str(" ");
        if let Some(expr) = &self.return_value {
            out.push_str(&expr.to_string());
        }
        out.push_str(";");
        out
    }
    
    fn node_type(&self) -> NodeType {
        NodeType::RETURN_STATEMENT
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Statement for ReturnStatement {
    fn statement_node(&self) {}
}

#[derive(Debug)]
pub struct Boolean {
    pub token: Rc<Token>,
    pub value: bool,
}

impl Node for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        self.token.literal.clone()
    }

    fn node_type(&self) -> NodeType {
        NodeType::BOOLEAN
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for Boolean {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct InfixExpression {
    pub token: Rc<Token>,
    pub left: Rc<dyn Expression>,
    pub operator: String,
    pub right: Rc<dyn Expression>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.to_string(),
            self.operator,
            self.right.to_string(),
        )
    }

    fn node_type(&self) -> NodeType {
        NodeType::INFIX_EXPRESSION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for InfixExpression {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct IfExpression {
    pub token: Rc<Token>,
    pub condition: Rc<dyn Expression>,
    pub consequence: Rc<dyn Statement>,
    pub alternative: Option<Rc<dyn Statement>>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("if");
        out.push_str(&self.condition.to_string());
        out.push_str(" ");
        out.push_str(&self.consequence.to_string());
        if let Some(alt) = &self.alternative {
            out.push_str(" else ");
            out.push_str(&alt.to_string());
        }
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::IF_EXPRESSION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for IfExpression {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct BlockStatement {
    pub token: Rc<Token>,
    pub statements: Vec<Rc<dyn Statement>>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("{");
        for s in &self.statements {
            out.push_str(&s.to_string());
        }
        out.push_str("}");
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::BLOCK_STATEMENT
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Statement for BlockStatement {
    fn statement_node(&self) {}
}

impl Clone for BlockStatement {
    fn clone(&self) -> Self {
        BlockStatement {
            token: self.token.clone(),
            statements: self.statements.clone(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Rc<Token>,
    pub parameters: Vec<Rc<Identifier>>,
    pub body: Rc<dyn Statement>,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.token_literal());
        out.push_str("(");
        for (i, p) in self.parameters.iter().enumerate() {
            out.push_str(&p.to_string());
            if i != self.parameters.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push_str(") ");
        out.push_str(&self.body.to_string());
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::FUNCTION_LITERAL
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for FunctionLiteral {
    fn expression_node(&self) {}
}

#[derive(Debug)]
pub struct CallExpression {
    pub token: Rc<Token>,
    pub function: Rc<dyn Expression>,
    pub arguments: Vec<Rc<dyn Expression>>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.function.to_string());
        out.push_str("(");
        for (i, arg) in self.arguments.iter().enumerate() {
            out.push_str(&arg.to_string());
            if i != self.arguments.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push_str(")");
        out
    }

    fn node_type(&self) -> NodeType {
        NodeType::CALL_EXPRESSION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Expression for CallExpression {
    fn expression_node(&self) {}
}
