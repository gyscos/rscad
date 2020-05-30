use crate::ast;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Id {
    /// How deeply scoped is the variable?
    /// 0: current scope
    /// 1: parent scope
    depth: usize,

    /// In the given scope, what is the ID?
    id: usize,
}

pub type VariableId = Id;
pub type FunctionId = Id;
pub type ModuleId = Id;

#[derive(Clone, Debug)]
pub enum Operator {}

#[derive(Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn from_str(field: &str) -> Option<Self> {
        Some(match field {
            "x" => Axis::X,
            "y" => Axis::Y,
            "z" => Axis::Z,
            _ => return None,
        })
    }
}

/// Represent a parsed expression.
#[derive(Clone, Debug)]
pub enum Expr {
    Undef,
    Extern,
    Boolean(bool),
    Number(f32),
    Text(String),
    Negative(Box<Expr>),
    Not(Box<Expr>),
    Variable(VariableId),
    Echo(Vec<ParameterValue>, Box<Expr>),
    Assert(Vec<ParameterValue>, Box<Expr>),
    Let(Vec<ParameterValue>, Box<Expr>),
    Function(FunctionId, Vec<ParameterValue>),
    ListComprehension {
        lets: Vec<Expr>,
        variables: Vec<ParameterValue>,
        body: Box<Expr>,
    },
    Vector(Vec<Expr>),
    Op(Box<Expr>, ast::Opcode, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    FieldAccess {
        parent: Box<Expr>,
        field: Axis,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        increment: Option<Box<Expr>>,
    },
}

#[derive(Clone, Debug)]
pub struct Function {
    default_values: Vec<Option<Expr>>,

    // only used for variables
    scope: Scope,

    body: Expr,
}

#[derive(Clone, Debug)]
pub struct Module {
    default_values: Vec<Option<Expr>>,
    body: Scope,
}

#[derive(Clone, Debug)]
pub struct ParameterValue {
    name: Option<String>,
    value: Expr,
}

#[derive(Clone, Debug)]
pub struct ParameterDefinition {
    default_value: Option<Expr>,
}

#[derive(Clone, Debug)]
enum ItemType {
    User(ModuleId),
    Extern(String),
}

#[derive(Clone, Debug)]
pub struct Item {
    ty: ItemType,
    params: Vec<Expr>,
    child: Scope,
}

#[derive(Clone, Debug, Default)]
pub struct Scope {
    variables: Vec<Expr>,
    functions: Vec<Function>,
    modules: Vec<Module>,
    items: Vec<Item>,
}

fn parse_parameter_value(parameter: ast::ParameterValue, context: &Context) -> ParameterValue {
    ParameterValue {
        name: parameter.name.map(str::to_string),
        value: parse_expr(parameter.value, context),
    }
}

fn parse_parameter_definitions(
    parameters: Vec<ast::ParameterDefinition>,
    context: &mut Context,
) -> Vec<Option<Expr>> {
    parameters
        .into_iter()
        .map(|param| {
            // Insert an entry in the scope variable (will be filled later)
            context.add_variable(param.name, |_| Expr::Extern);

            // And save the default value in the parent context
            param
                .default_value
                .map(|v| parse_expr(v, context.parent.unwrap()))
        })
        .collect()
}

fn parse_parameter_values<'a>(
    parameters: Vec<ast::ParameterValue>,
    context: &Context,
) -> Vec<ParameterValue> {
    parameters
        .into_iter()
        .map(|p| parse_parameter_value(p, context))
        .collect()
}

fn parse_expr<'a>(expr: ast::Expr, context: &Context<'a>) -> Expr {
    // Define handy lambdas to avoid repeating the context
    let parse_expr = |expr: ast::Expr| parse_expr(expr, context);
    let parse_boxed_expr = |expr: Box<ast::Expr>| Box::new(parse_expr(*expr));

    match expr {
        ast::Expr::Undef => Expr::Undef,
        ast::Expr::Boolean(b) => Expr::Boolean(b),
        ast::Expr::Number(n) => Expr::Number(n),
        ast::Expr::Text(text) => Expr::Text(text.to_string()),
        ast::Expr::Vector(values) => Expr::Vector(values.into_iter().map(parse_expr).collect()),
        ast::Expr::Variable(var) => {
            context
                .find_var(var)
                .map(Expr::Variable)
                .unwrap_or_else(|| {
                    eprintln!("Could not find variable `{}`", var);
                    Expr::Undef
                })
        }
        ast::Expr::Function(ast::FunctionCall { name, parameters }) => context
            .find_function(name)
            .map(|fid| Expr::Function(fid, parse_parameter_values(parameters, context)))
            .unwrap_or_else(|| {
                log::warn!("Could not find function `{}`", name);
                Expr::Undef
            }),
        ast::Expr::Negative(expr) => Expr::Negative(parse_boxed_expr(expr)),
        ast::Expr::Not(expr) => Expr::Not(parse_boxed_expr(expr)),
        ast::Expr::Echo(params, expr) => Expr::Echo(
            parse_parameter_values(params, context),
            parse_boxed_expr(expr),
        ),
        ast::Expr::Assert(params, expr) => Expr::Assert(
            parse_parameter_values(params, context),
            parse_boxed_expr(expr),
        ),
        ast::Expr::Let(lets, expr) => Expr::Let(
            lets.into_iter()
                .flat_map(|params| parse_parameter_values(params.vars, context))
                .collect(),
            parse_boxed_expr(expr),
        ),
        ast::Expr::Or(a, b) => Expr::Or(parse_boxed_expr(a), parse_boxed_expr(b)),
        ast::Expr::And(a, b) => Expr::And(parse_boxed_expr(a), parse_boxed_expr(b)),
        ast::Expr::Op(a, op, b) => Expr::Op(parse_boxed_expr(a), op, parse_boxed_expr(b)),
        ast::Expr::FieldAccess { parent, field } => Axis::from_str(field)
            .map(|field| Expr::FieldAccess {
                parent: parse_boxed_expr(parent),
                field,
            })
            .unwrap_or_else(|| {
                log::warn!("Unrecognized field access `{}`", field);
                Expr::Undef
            }),
        ast::Expr::ArrayAccess { array, index } => Expr::ArrayAccess {
            array: parse_boxed_expr(array),
            index: parse_boxed_expr(index),
        },
        ast::Expr::Ternary {
            condition,
            if_true,
            if_false,
        } => Expr::Ternary {
            condition: parse_boxed_expr(condition),
            if_true: parse_boxed_expr(if_true),
            if_false: parse_boxed_expr(if_false),
        },
        ast::Expr::Range {
            start,
            end,
            increment,
        } => Expr::Range {
            start: parse_boxed_expr(start),
            end: parse_boxed_expr(end),
            increment: increment.map(parse_boxed_expr),
        },
        ast::Expr::ListComprehension {
            lets,
            variables,
            body,
        } => Expr::ListComprehension {
            lets: Vec::new(),
            variables: parse_parameter_values(variables, context),
            body: parse_boxed_expr(body),
        },
    }
}

fn parse_statement<'a>(statement: ast::Statement, context: &mut Context<'a>) {
    match statement {
        ast::Statement::VariableDeclaration(name, expr) => {
            // Insert a new variable in scope
            context.add_variable(name, |context| parse_expr(expr, context));
        }
        ast::Statement::ModuleDefinition { name, args, body } => {
            context.add_module(name, |context| {
                let mut context = Context::new(context);
                // For each param:
                let default_values = parse_parameter_definitions(args, &mut context);

                // Insert the module parameters in the child context
                parse_scope(*body, &mut context);

                Module {
                    default_values,
                    body: context.scope,
                }
            });
        }
        ast::Statement::FunctionDefinition(name, params, body) => {
            context.add_function(name, |context| {
                let mut context = Context::new(context);

                let default_values = parse_parameter_definitions(params, &mut context);
                let body = parse_expr(body, &context);
                let scope = context.scope;

                Function {
                    default_values,
                    body,
                    scope,
                }
            });
        }
        _ => (),
    }
}

fn parse_scope<'a>(statement: ast::Statement, context: &mut Context<'a>) {
    // Step 1: check all Module, Function and variable names.
    // No value / default value
    // Ignore module calls/ifs for now

    parse_statement(statement, context);

    // Step 2: resolve the actual values and sub-scopes
}

struct Context<'a> {
    scope: Scope,
    variables_map: HashMap<String, usize>,
    functions_map: HashMap<String, usize>,
    modules_map: HashMap<String, usize>,

    parent: Option<&'a Context<'a>>,
}

impl<'a> Context<'a> {
    fn new(parent: &'a Context<'a>) -> Self {
        Context {
            scope: Scope::default(),
            parent: Some(parent),
            variables_map: HashMap::new(),
            functions_map: HashMap::new(),
            modules_map: HashMap::new(),
        }
    }

    fn add_function<F: FnOnce(&Self) -> Function>(&mut self, name: &str, f: F) {
        let id = self.scope.functions.len();
        self.functions_map.insert(name.to_string(), id);
        let function = f(self);
        self.scope.functions.push(function);
    }

    fn add_module<F: FnOnce(&Self) -> Module>(&mut self, name: &str, f: F) {
        let id = self.scope.modules.len();
        self.modules_map.insert(name.to_string(), id);
        let module = f(self);
        self.scope.modules.push(module);
    }

    fn add_variable<F: FnOnce(&Self) -> Expr>(&mut self, name: &str, f: F) {
        let id = self.scope.variables.len();
        self.variables_map.insert(name.to_string(), id);
        let variable = f(self);
        self.scope.variables.push(variable);
    }

    fn find_var(&self, name: &str) -> Option<VariableId> {
        find_id(name, self, |s| &s.variables_map)
    }

    fn find_module(&self, name: &str) -> Option<ModuleId> {
        find_id(name, self, |s| &s.modules_map)
    }

    fn find_function(&self, name: &str) -> Option<FunctionId> {
        find_id(name, self, |s| &s.functions_map)
    }
}

fn add<T, F: FnOnce(&Context) -> T>(
    name: &str,
    map: &mut HashMap<String, T>,
    values: &mut Vec<T>,
    f: F,
) {
}

fn find_id<F>(name: &str, context: &Context, f: F) -> Option<Id>
where
    F: for<'a> Fn(&'a Context) -> &'a HashMap<String, usize>,
{
    f(context)
        .get(name)
        .map(|&id| Id { depth: 0, id })
        .or_else(|| {
            context.parent.and_then(|parent| {
                find_id(name, parent, f).map(|Id { depth, id }| Id {
                    depth: depth + 1,
                    id,
                })
            })
        })
}

pub fn parse(statements: &[ast::Statement]) -> Scope {
    Scope {
        variables: vec![],
        functions: vec![],
        modules: vec![],
        items: vec![],
    }
}
