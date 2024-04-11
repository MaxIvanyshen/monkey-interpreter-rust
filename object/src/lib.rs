use std::{cell::RefCell, fmt::{Debug, Formatter}, rc::Rc};

use ast::Node;

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    INTEGER,
    BOOLEAN,
    NULL,
    ERROR,
    RETURN_VALUE,
    FUNCTION,
    IDENTIFIER,
    STRING,
}

impl Debug for dyn Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

pub trait Object {
    fn object_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct StringObj {
    pub value: String,
}

impl Object for StringObj {
    fn object_type(&self) -> ObjectType {
        ObjectType::STRING
    }

    fn inspect(&self) -> String {
        self.value.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Integer {
    pub value: i64,
}

impl Object for Integer {
    fn object_type(&self) -> ObjectType {
        ObjectType::INTEGER
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Boolean {
    pub value: bool,
}

impl Object for Boolean {
    fn object_type(&self) -> ObjectType {
        ObjectType::BOOLEAN
    }

    fn inspect(&self) -> String {
        self.value.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Null {}

impl Object for Null {
    fn object_type(&self) -> ObjectType {
        ObjectType::NULL
    }

    fn inspect(&self) -> String {
        "null".to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Error {
    pub message: String,
}

impl Object for Error {
    fn object_type(&self) -> ObjectType {
        ObjectType::ERROR
    }

    fn inspect(&self) -> String {
        self.message.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ReturnValue {
    pub value: Rc<dyn Object>,
}

impl Object for ReturnValue {
    fn object_type(&self) -> ObjectType {
        ObjectType::RETURN_VALUE
    }

    fn inspect(&self) -> String {
        self.value.inspect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Function {
    pub parameters: Vec<Rc<ast::Identifier>>,
    pub body: Rc<dyn ast::Statement>,
    pub env: Rc<RefCell<Environment>>,
}

impl Object for Function {
    fn object_type(&self) -> ObjectType {
        ObjectType::FUNCTION
    }

    fn inspect(&self) -> String {
        let mut out = String::new();
        out.push_str("fn(");
        for p in &self.parameters {
            out.push_str(&p.value);
            out.push_str(", ");
        }
        out.push_str(") {\n");
        out.push_str(&self.body.to_string());
        out.push_str("\n}");
        out
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Environment {
    pub outer : Option<Rc<Environment>>,
    pub scope: std::collections::HashMap<String, Rc<dyn Object>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            outer: None,
            scope: std::collections::HashMap::new(),
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let mut env = Environment::new();
        env.scope = outer.borrow().scope.clone();
        Rc::new(RefCell::new(env))
    }

    pub fn get(&self, name: &str) -> Option<Rc<dyn Object>> {
        match self.scope.get(name) {
            Some(obj) => Some(obj.clone()),
            None => None,
        }
    }

    pub fn set(&mut self, name: String, value: Rc<dyn Object>) -> Option<Rc<dyn Object>> {
        self.scope.insert(name, value)
    }
}
