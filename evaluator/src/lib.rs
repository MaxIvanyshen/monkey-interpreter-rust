use std::rc::Rc;
use std::cell::RefCell;

pub fn evaluate_program(program: ast::Program, env: Rc<RefCell<object::Environment>>) -> Option<Rc<dyn object::Object>> {
    let mut result = None;
    for statement in program.statements {
        let evaluated = evaluate_statement(statement, env.clone());
        match evaluated.object_type() {
            object::ObjectType::RETURN_VALUE => {
                result = Some(evaluated.as_ref().as_any().downcast_ref::<object::ReturnValue>().unwrap().value.clone());
                break;
            }
            object::ObjectType::ERROR => {
                result = Some(Rc::new(object::Error { message: evaluated.as_ref().as_any().downcast_ref::<object::Error>().unwrap().message.clone() }));
                break;
            }
            _ => { result = Some(evaluated);}
        }
    }
    result
}

fn evaluate_statement(statement: Rc<dyn ast::Statement>, env: Rc<RefCell<object::Environment>>) -> Rc<dyn object::Object> {
    match statement.node_type() {
        ast::NodeType::EXPRESSION_STATEMENT => {
            let expression = statement.as_ref().as_any().downcast_ref::<ast::ExpressionStatement>().unwrap().expression.as_ref().unwrap().clone();
            evaluate_expression(expression, env)
        },
        ast::NodeType::LET_STATEMENT => {
            let let_statement = statement.as_ref().as_any().downcast_ref::<ast::LetStatement>().unwrap();
            let value = evaluate_expression(let_statement.value.as_ref().unwrap().clone(), env.clone());
            if value.object_type() == object::ObjectType::ERROR {
                return value;
            }
            env.borrow_mut().set(let_statement.name.value.clone(), value);
            Rc::new(object::Null {})
        },
        ast::NodeType::RETURN_STATEMENT => {
            let return_statement = statement.as_ref().as_any().downcast_ref::<ast::ReturnStatement>().unwrap();
            let value = evaluate_expression(return_statement.return_value.as_ref().unwrap().clone(), env);
            if value.object_type() == object::ObjectType::ERROR {
                return value;
            }
            Rc::new(object::ReturnValue { value })
        },  
        ast::NodeType::BLOCK_STATEMENT => {
            let block_env = object::Environment::new_enclosed(env);
            let result = evaluate_block_statement(statement, block_env);
            result
        },
        _ => Rc::new(object::Null {})
    }
}

fn evaluate_expression(exp: Rc<dyn ast::Expression>, env: Rc<RefCell<object::Environment>>) -> Rc<dyn object::Object> {
    match exp.node_type() {
        ast::NodeType::IDENTIFIER => {
            let identifier = exp.as_ref().as_any().downcast_ref::<ast::Identifier>().unwrap();
            match env.borrow().get(identifier.value.as_str()) {
                Some(obj) => obj,
                None => Rc::new(object::Error { message: format!("identifier not found: {}", identifier.value) })
            }
        },
        ast::NodeType::INTEGER_LITERAL => {
            let integer = exp.as_ref().as_any().downcast_ref::<ast::IntegerLiteral>().unwrap();
            Rc::new(object::Integer { value: integer.value })
        },
        ast::NodeType::STRING_LITERAL => {
            let string = exp.as_ref().as_any().downcast_ref::<ast::StringLiteral>().unwrap();
            Rc::new(object::StringObj { value: string.value.clone() })
        },
        ast::NodeType::BOOLEAN => {
            let boolean = exp.as_ref().as_any().downcast_ref::<ast::Boolean>().unwrap();
            if boolean.value {
                Rc::new(object::Boolean { value: true })
            } else {
                Rc::new(object::Boolean { value: false })
            }
        },
        ast::NodeType::PREFIX_EXPRESSION => {
            let prefix = exp.as_ref().as_any().downcast_ref::<ast::PrefixExpression>().unwrap();
            let right = evaluate_expression(prefix.right.clone(), env);
            if right.object_type() == object::ObjectType::ERROR {
                return right;
            }
            evaluate_prefix_expression(prefix.operator.as_str(), right)
        },
        ast::NodeType::INFIX_EXPRESSION => {
            let infix = exp.as_ref().as_any().downcast_ref::<ast::InfixExpression>().unwrap();
            let left = evaluate_expression(infix.left.clone(), env.clone());
            if left.object_type() == object::ObjectType::ERROR {
                return left;
            }
            let right = evaluate_expression(infix.right.clone(), env.clone());
            if right.object_type() == object::ObjectType::ERROR {
                return right;
            }
            evaluate_infix_expression(infix.operator.as_str(), left, right)
        },
        ast::NodeType::EXPRESSION_STATEMENT => {
            let expression = exp.as_ref().as_any().downcast_ref::<ast::ExpressionStatement>().unwrap().expression.as_ref().unwrap().clone();
            evaluate_expression(expression, env)
        },
        ast::NodeType::IF_EXPRESSION => {
            let if_expression = exp.as_ref().as_any().downcast_ref::<ast::IfExpression>().unwrap();
            let condition = evaluate_expression(if_expression.condition.clone(), env.clone());
            if condition.object_type() == object::ObjectType::ERROR {
                return condition;
            }

            if is_truthy(condition) {
                let result = evaluate_block_statement(if_expression.consequence.clone(), env);
                result
            } else if let Some(alternative) = if_expression.alternative.clone() {
                evaluate_block_statement(alternative, env.clone())
            } else {
                Rc::new(object::Null {})
            }
        },
        ast::NodeType::FUNCTION_LITERAL => {
            let function_literal = exp.as_ref().as_any().downcast_ref::<ast::FunctionLiteral>().unwrap();
            Rc::new(object::Function { parameters: function_literal.parameters.clone(), body: function_literal.body.clone(), 
                env: env.clone() })
        },
        ast::NodeType::CALL_EXPRESSION => {
            let call_expression = exp.as_ref().as_any().downcast_ref::<ast::CallExpression>().unwrap();
            let function = evaluate_expression(call_expression.function.clone(), env.clone());
            if function.object_type() == object::ObjectType::ERROR {
                return function;
            }
            let args = evaluate_expressions(call_expression.arguments.clone(), env.clone());
            if args.len() == 1 && args[0].object_type() == object::ObjectType::ERROR {
                return args[0].clone();
            }
            apply_function(function, args)
        },
        _ => Rc::new(object::Null {})
    }
}

fn evaluate_prefix_expression(operator: &str, right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    match operator {
        "!" => evaluate_bang_operator_expression(right),
        "-" => evaluate_minus_prefix_operator_expression(right),
        _ => Rc::new(object::Null {})
    }
}

fn evaluate_bang_operator_expression(right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    match right.object_type() {
        object::ObjectType::BOOLEAN => {
            let boolean = right.as_ref().as_any().downcast_ref::<object::Boolean>().unwrap();
            if boolean.value {
                Rc::new(object::Boolean { value: false })
            } else {
                Rc::new(object::Boolean { value: true })
            }
        },
        object::ObjectType::NULL => Rc::new(object::Boolean { value: true }),
        _ => Rc::new(object::Boolean { value: false })
    }
}

fn evaluate_minus_prefix_operator_expression(right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    match right.object_type() {
        object::ObjectType::INTEGER => {
            let integer = right.as_ref().as_any().downcast_ref::<object::Integer>().unwrap();
            Rc::new(object::Integer { value: -integer.value })
        },
        _ => Rc::new(object::Error { message: format!("unknown operator: -{:?}", right.object_type()) })
    }
}

fn evaluate_infix_expression(operator: &str, left: Rc<dyn object::Object>, right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    if left.object_type() == object::ObjectType::STRING && right.object_type() == object::ObjectType::STRING && operator == "+" {
        return evaluate_string_concatenation(left, right);
    }
    if left.object_type() == object::ObjectType::INTEGER && right.object_type() == object::ObjectType::INTEGER {
        return evaluate_integer_infix_expression(operator, left, right);
    }
    if left.object_type() == object::ObjectType::BOOLEAN && right.object_type() == object::ObjectType::BOOLEAN {
        return evaluate_boolean_infix_expression(operator, left, right);
    }
    if left.object_type() != right.object_type() {
        return Rc::new(object::Error { message: format!("type mismatch: {:?} {} {:?}", left.object_type(), operator, right.object_type()) });
    }
    Rc::new(object::Error { message: format!("unknown operator: {:?} {} {:?}", left.object_type(), operator, right.object_type()) })
}

fn evaluate_integer_infix_expression(operator: &str, left: Rc<dyn object::Object>, right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    let left_integer = left.as_ref().as_any().downcast_ref::<object::Integer>().unwrap();
    let right_integer = right.as_ref().as_any().downcast_ref::<object::Integer>().unwrap();
    match operator {
        "+" => Rc::new(object::Integer { value: left_integer.value + right_integer.value }),
        "-" => Rc::new(object::Integer { value: left_integer.value - right_integer.value }),
        "*" => Rc::new(object::Integer { value: left_integer.value * right_integer.value }),
        "/" => Rc::new(object::Integer { value: left_integer.value / right_integer.value }),
        "<" => Rc::new(object::Boolean { value: left_integer.value < right_integer.value }),
        ">" => Rc::new(object::Boolean { value: left_integer.value > right_integer.value }),
        "==" => Rc::new(object::Boolean { value: left_integer.value == right_integer.value }),
        "!=" => Rc::new(object::Boolean { value: left_integer.value != right_integer.value }),
        "%" => Rc::new(object::Integer {value: left_integer.value % right_integer.value }),
        _ => Rc::new(object::Error { message: format!("unknown operator: {:?} {} {:?}", left.object_type(), operator, right.object_type()) })
    }
}

fn evaluate_string_concatenation(left: Rc<dyn object::Object>, right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    let left_string = left.as_ref().as_any().downcast_ref::<object::StringObj>().unwrap();
    let right_string = right.as_ref().as_any().downcast_ref::<object::StringObj>().unwrap();
    Rc::new(object::StringObj { value: format!("{}{}", left_string.value, right_string.value) })
}

fn evaluate_boolean_infix_expression(operator: &str, left: Rc<dyn object::Object>, right: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    let left_boolean = left.as_ref().as_any().downcast_ref::<object::Boolean>().unwrap();
    let right_boolean = right.as_ref().as_any().downcast_ref::<object::Boolean>().unwrap();
    match operator {
        "==" => Rc::new(object::Boolean { value: left_boolean.value == right_boolean.value }),
        "!=" => Rc::new(object::Boolean { value: left_boolean.value != right_boolean.value }),
        _ => Rc::new(object::Error { message: format!("unknown operator: {:?} {} {:?}", left.object_type(), operator, right.object_type()) })
    }
}

fn evaluate_block_statement(stmt: Rc<dyn ast::Statement>, env: Rc<RefCell<object::Environment>>) -> Rc<dyn object::Object> {
    let block = stmt.as_ref().as_any().downcast_ref::<ast::BlockStatement>().unwrap();
    let mut result = evaluate_statement(block.statements.first().unwrap().clone(), env.clone());
    for statement in block.statements.iter() {
        let evaluated = evaluate_statement(statement.clone(), env.clone());
        match evaluated.object_type() {
            object::ObjectType::RETURN_VALUE => return evaluated,
            object::ObjectType::ERROR => return evaluated,
            _ => { result = evaluated;}
        }
    }
    result
}

fn is_truthy(obj: Rc<dyn object::Object>) -> bool {
    match obj.object_type() {
        object::ObjectType::NULL => false,
        object::ObjectType::BOOLEAN => {
            let boolean = obj.as_ref().as_any().downcast_ref::<object::Boolean>().unwrap();
            boolean.value
        },
        _ => true
    }
}

fn apply_function(func: Rc<dyn object::Object>, args: Vec<Rc<dyn object::Object>>) -> Rc<dyn object::Object> {
    match func.object_type() {
        object::ObjectType::FUNCTION => {
            let function = func.as_ref().as_any().downcast_ref::<object::Function>().unwrap();
            let extended_env = extend_function_env(function, args);
            let evaluated = evaluate_statement(function.body.clone(), extended_env);
            unwrap_return_value(evaluated)
        },
        _ => Rc::new(object::Error { message: format!("not a function: {:?}", func.object_type()) })
    }
}

fn extend_function_env(func: &object::Function, args: Vec<Rc<dyn object::Object>>) -> Rc<RefCell<object::Environment>> {
    let env = object::Environment::new_enclosed(func.env.clone());
    for (i, param) in func.parameters.iter().enumerate() {
        env.borrow_mut().set(param.value.clone(), args[i].clone());
    }
    env
}

fn unwrap_return_value(obj: Rc<dyn object::Object>) -> Rc<dyn object::Object> {
    if obj.object_type() == object::ObjectType::RETURN_VALUE {
        return obj.as_ref().as_any().downcast_ref::<object::ReturnValue>().unwrap().value.clone();
    }
    obj
}

fn evaluate_expressions(exps: Vec<Rc<dyn ast::Expression>>, env: Rc<RefCell<object::Environment>>) -> Vec<Rc<dyn object::Object>> {
    let mut result = Vec::new();
    for exp in exps {
        let evaluated = evaluate_expression(exp, env.clone());
        if evaluated.object_type() == object::ObjectType::ERROR {
            return vec![evaluated];
        }
        result.push(evaluated);
    }
    result
}
