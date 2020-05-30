//! Raw AST for the OpenSCAD syntax.
//!
//! This does not know about any standard functions (like `sphere`, `import`, or even `if`  and `for`).

/// An item in a SCAD scene.
#[derive(Clone, Debug)]
pub enum Statement<'input> {
    /// Variable declaration
    VariableDeclaration(&'input str, Expr<'input>),

    /// A statement block in { }
    StatementList(Vec<Statement<'input>>),

    /// Nothing.
    NoOp,

    /// Module definition
    ModuleDefinition {
        name: &'input str,
        args: Vec<ParameterDefinition<'input>>,
        body: Box<Statement<'input>>,
    },

    /// Function definition
    FunctionDefinition(&'input str, Vec<ParameterDefinition<'input>>, Expr<'input>),

    /// Includes another file
    Include(&'input str),

    /// Use another file
    Use(&'input str),

    /// Function call (or module call)
    ModuleCall(ModuleCall<'input>),

    /// A for-loop
    For {
        /// Variables looped over
        variables: Vec<ParameterValue<'input>>,
        /// Body of the for-loop
        body: Vec<Statement<'input>>,
        /// Optional modifier
        modifier: Option<Modifier>,
    },

    /// A comment (can be ignored)
    Comment(&'input str),

    /// If-block
    If {
        /// Condition for this block
        condition: Expr<'input>,
        /// Body if the condition is true
        if_true: Vec<Statement<'input>>,
        /// Body if the condition is false
        if_false: Vec<Statement<'input>>,
    },
}

impl<'input> Statement<'input> {
    pub(crate) fn make_if(condition: Expr<'input>, if_true: Vec<Statement<'input>>) -> Self {
        Statement::If {
            condition,
            if_true,
            if_false: vec![],
        }
    }

    pub(crate) fn make_if_else(
        condition: Expr<'input>,
        if_true: Vec<Statement<'input>>,
        if_false: Statement<'input>,
    ) -> Self {
        let if_false = vec![if_false];
        Statement::If {
            condition,
            if_true,
            if_false,
        }
    }
}

/// Describes a function call: ex `sphere(1, center=true)`
#[derive(Clone, Debug)]
pub struct ModuleCall<'input> {
    /// Name of the function being called
    pub function: &'input str,
    /// List of parameters given
    pub params: Vec<ParameterValue<'input>>,
    /// Children of the call, if any (used for `union`/`difference`/...)
    pub children: Vec<Statement<'input>>,

    /// Optional modifier
    pub modifier: Option<Modifier>,
}

#[derive(Clone, Copy, Debug)]
pub enum Modifier {
    /// Do not render this element
    Disable,
    /// Only render this element
    ShowOnly,
    /// Highlight this element
    Highlight,
    /// Make this element transparent
    Transparent,
}

/// A parameter given to a function, possibly named.
#[derive(Clone, Debug)]
pub struct ParameterValue<'input> {
    /// Optional name for this parameter
    pub name: Option<&'input str>,
    /// Value given to this parameter
    pub value: Expr<'input>,
}

/// An argument in a function declaration, possibly with default value.
#[derive(Clone, Debug)]
pub struct ParameterDefinition<'input> {
    /// Name of the parameter
    pub name: &'input str,
    /// Optional default value for this parameter
    pub default_value: Option<Expr<'input>>,
}

#[derive(Clone, Debug)]
pub struct FunctionCall<'input> {
    /// Name of the function being called
    pub name: &'input str,
    /// Parameters given to the function
    pub parameters: Vec<ParameterValue<'input>>,
}

/// A local variable definition: `let (a=42)`
#[derive(Clone, Debug)]
pub struct Let<'input> {
    pub vars: Vec<ParameterValue<'input>>,
}

/// An expression in the AST. Directly what lalrpop produces.
#[derive(Clone, Debug)]
pub enum Expr<'input> {
    /// Undefined expression.
    Undef,
    /// A boolean literal
    Boolean(bool),
    /// A number literal
    Number(f32),
    /// A text literal
    Text(&'input str),
    /// Negative another expression
    Negative(Box<Expr<'input>>),
    /// Boolean NOT (!)
    Not(Box<Expr<'input>>),
    /// A variable
    Variable(&'input str),
    /// A function call
    Function(FunctionCall<'input>),
    /// Print something, the resolve the expression.
    Echo(Vec<ParameterValue<'input>>, Box<Expr<'input>>),
    /// Print something, the resolve the expression.
    Assert(Vec<ParameterValue<'input>>, Box<Expr<'input>>),
    /// Defines some local variables, then resolve the expression.
    Let(Vec<Let<'input>>, Box<Expr<'input>>),
    /// A list comprehension: [let(n=5) for(i = [1:n]) i*i]
    ListComprehension {
        /// Variable definitions to run before the loop.
        lets: Vec<Let<'input>>,
        /// Variables to loop over
        variables: Vec<ParameterValue<'input>>,
        /// Body of the loop
        body: Box<Expr<'input>>,
    },
    /// A vector: `[1, 2, 3*a]`
    Vector(Vec<Expr<'input>>),
    /// An operation: `a + 3`, `f(n) == 0`, ...
    Op(Box<Expr<'input>>, Opcode, Box<Expr<'input>>),
    /// `a && b`
    Or(Box<Expr<'input>>, Box<Expr<'input>>),
    /// `a || b`
    And(Box<Expr<'input>>, Box<Expr<'input>>),
    /// Access a field from an object: `foobar.x`
    FieldAccess {
        parent: Box<Expr<'input>>,
        field: &'input str,
    },
    /// Access an array value: `a[2 + 3]`
    ArrayAccess {
        /// Array to index into
        array: Box<Expr<'input>>,
        /// Index value
        index: Box<Expr<'input>>,
    },
    /// A ternary operation: `a ? b : c`
    Ternary {
        /// Condition of the ternary
        condition: Box<Expr<'input>>,
        /// Body if the condition is true
        if_true: Box<Expr<'input>>,
        /// Body if the condition is false
        if_false: Box<Expr<'input>>,
    },
    /// A range: `[0 : 10 : 100]`
    Range {
        /// Inclusive start of the range
        start: Box<Expr<'input>>,
        /// Inclusive end of the range
        end: Box<Expr<'input>>,
        /// Increment this value each step
        increment: Option<Box<Expr<'input>>>,
    },
}

impl<'input> Expr<'input> {
    pub(crate) fn array_access(array: Expr<'input>, index: Expr<'input>) -> Self {
        let array = Box::new(array);
        let index = Box::new(index);
        Expr::ArrayAccess { array, index }
    }

    pub(crate) fn field_access(parent: Expr<'input>, field: &'input str) -> Self {
        let parent = Box::new(parent);
        Expr::FieldAccess { parent, field }
    }

    pub(crate) fn range(start: Self, increment: Option<Self>, end: Self) -> Self {
        let start = Box::new(start);
        let increment = increment.map(Box::new);
        let end = Box::new(end);
        Expr::Range {
            start,
            increment,
            end,
        }
    }
}

/// An operation between expressions
#[derive(Clone, Debug)]
pub enum Opcode {
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Remainder
    Rem,
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Equality check
    Equal,
    /// Not-equality check
    NotEqual,
    /// Greater than
    Gt,
    /// Greater than, or equal
    Gte,
    /// Less than
    Lt,
    /// Less than, or equal
    Lte,
}
