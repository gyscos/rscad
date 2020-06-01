use rscad::ast;
use rscad::parse;

fn cube<'a>() -> ast::Statement<'a> {
    ast::Statement::ModuleCall(ast::ModuleCall {
        function: "cube",
        params: vec![ast::ParameterValue {
            name: None,
            value: ast::Expr::Vector(vec![
                ast::Expr::Number(1f32),
                ast::Expr::Number(2f32),
                ast::Expr::Number(3f32),
            ]),
        }],
        child: Box::new(ast::Statement::NoOp),
    })
}

#[test]
fn fail_on_bad_modifiers() {
    assert!(
        parse("% { cube([1,2,3]); }").is_err(),
        "cannot apply modifier to a list of items",
    );

    assert!(
        parse("% a = 0;").is_err(),
        "cannot apply modifier to a variable assignment"
    );

    assert!(
        parse("% module foo() { }").is_err(),
        "cannot apply modifier to a module definition",
    );

    assert!(
        parse("% ;").is_err(),
        "cannot apply modifier to no-op statement"
    );
}

#[test]
fn parse_modifiers() {
    assert_eq!(
        parse("%cube([1,2,3]);").unwrap(),
        vec![ast::Statement::Modifier(
            ast::Modifier::Transparent,
            Box::new(cube()),
        )],
    );
}

#[test]
fn parse_module_call() {
    assert_eq!(parse("cube([1,2,3]);").unwrap(), vec![cube()],);
}

#[test]
fn parse_module_child() {
    assert_eq!(
        parse(
            r#"
            translate([1,2,3]) cube([4,5,6]);
            "#
        )
        .unwrap(),
        vec![ast::Statement::ModuleCall(ast::ModuleCall {
            function: "translate",
            params: vec![ast::ParameterValue {
                name: None,
                value: ast::Expr::Vector(vec![
                    ast::Expr::Number(1.0),
                    ast::Expr::Number(2.0),
                    ast::Expr::Number(3.0),
                ]),
            }],
            child: Box::new(ast::Statement::ModuleCall(ast::ModuleCall {
                function: "cube",
                params: vec![ast::ParameterValue {
                    name: None,
                    value: ast::Expr::Vector(vec![
                        ast::Expr::Number(4.0),
                        ast::Expr::Number(5.0),
                        ast::Expr::Number(6.0),
                    ]),
                }],
                child: Box::new(ast::Statement::NoOp),
            })),
        })],
    );
}

#[test]
fn parse_module_definition() {
    assert_eq!(
        parse(
            r#"
            module foo() {
                cube([1,2,3]);
            }
            "#,
        )
        .unwrap(),
        vec![ast::Statement::ModuleDefinition {
            name: "foo",
            args: vec![],
            body: Box::new(ast::Statement::StatementList(vec![cube()])),
        }],
    );
}

#[test]
fn parse_empty() {
    assert_eq!(parse("").unwrap(), vec![],);
}

#[test]
fn parse_comments() {
    assert_eq!(
        parse(
            r#"
            // First comment

            /* Second comment */

            /* No /* nested comments */

            /* Multi-line
             * Comment */

            /*
            // Easy-to-toggle comment
            // */
            "#
        )
        .unwrap(),
        vec![],
    );
}

#[test]
fn parse_list_comprehension() {
    assert_eq!(
        parse(
            r#"
            a = [let(n=5) for(i=[1:n]) i*i];
            "#
        )
        .unwrap(),
        vec![ast::Statement::VariableDeclaration(
            "a",
            ast::Expr::ListComprehension {
                lets: vec![ast::Let {
                    vars: vec![ast::ParameterValue {
                        name: Some("n"),
                        value: ast::Expr::Number(5.0),
                    }],
                }],
                variables: vec![ast::ParameterValue {
                    name: Some("i"),
                    value: ast::Expr::Range {
                        start: Box::new(ast::Expr::Number(1.0)),
                        end: Box::new(ast::Expr::Variable("n")),
                        increment: None,
                    },
                }],
                body: Box::new(ast::Expr::Op(
                    Box::new(ast::Expr::Variable("i")),
                    ast::Opcode::Mul,
                    Box::new(ast::Expr::Variable("i")),
                )),
            }
        )],
    );
}

#[test]
fn fail_on_nested_comments() {
    assert!(parse(" /*  This /* nested */ comment should fail. */ ").is_err());
}

#[test]
fn parse_ternary() {
    assert_eq!(
        parse("a = 1 > 2 ? 1 + 2 : 3;").unwrap(),
        vec![ast::Statement::VariableDeclaration(
            "a",
            ast::Expr::Ternary {
                condition: Box::new(ast::Expr::Op(
                    Box::new(ast::Expr::Number(1.0)),
                    ast::Opcode::Gt,
                    Box::new(ast::Expr::Number(2.0)),
                )),
                if_true: Box::new(ast::Expr::Op(
                    Box::new(ast::Expr::Number(1.0)),
                    ast::Opcode::Add,
                    Box::new(ast::Expr::Number(2.0)),
                )),
                if_false: Box::new(ast::Expr::Number(3.0)),
            }
        )],
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        parse(
            r#"
            a = [1,2,3];
            b = a[0];
            "#,
        )
        .unwrap(),
        vec![
            ast::Statement::VariableDeclaration(
                "a",
                ast::Expr::Vector(vec![
                    ast::Expr::Number(1.0),
                    ast::Expr::Number(2.0),
                    ast::Expr::Number(3.0),
                ]),
            ),
            ast::Statement::VariableDeclaration(
                "b",
                ast::Expr::ArrayAccess {
                    array: Box::new(ast::Expr::Variable("a")),
                    index: Box::new(ast::Expr::Number(0.0)),
                },
            )
        ]
    );
}

#[test]
fn parse_field_access() {
    assert_eq!(
        parse(
            r#"
            a = [1,2,3];
            b = a.x;
            "#
        )
        .unwrap(),
        vec![
            ast::Statement::VariableDeclaration(
                "a",
                ast::Expr::Vector(vec![
                    ast::Expr::Number(1.0),
                    ast::Expr::Number(2.0),
                    ast::Expr::Number(3.0),
                ]),
            ),
            ast::Statement::VariableDeclaration(
                "b",
                ast::Expr::FieldAccess {
                    parent: Box::new(ast::Expr::Variable("a")),
                    field: "x",
                },
            )
        ]
    );
}

#[test]
fn parse_boolean() {
    assert_eq!(
        parse(
            r#"
            a = true || !(false && true);
            "#
        )
        .unwrap(),
        vec![ast::Statement::VariableDeclaration(
            "a",
            ast::Expr::Or(
                Box::new(ast::Expr::Boolean(true)),
                Box::new(ast::Expr::Not(Box::new(ast::Expr::And(
                    Box::new(ast::Expr::Boolean(false)),
                    Box::new(ast::Expr::Boolean(true)),
                )))),
            ),
        )],
    );
}

#[test]
fn parse_translate_child() {
    assert_eq!(
        parse(
            r#"
                translate([1,2,3]) {
                    a = 5;
                    { cube([a,a,a]); }
                }
            "#
        )
        .unwrap(),
        vec![ast::Statement::ModuleCall(ast::ModuleCall {
            function: "translate",
            params: vec![ast::ParameterValue {
                name: None,
                value: ast::Expr::Vector(vec![
                    ast::Expr::Number(1.0),
                    ast::Expr::Number(2.0),
                    ast::Expr::Number(3.0),
                ]),
            }],
            child: Box::new(ast::Statement::StatementList(vec![
                ast::Statement::VariableDeclaration("a", ast::Expr::Number(5.0),),
                ast::Statement::StatementList(vec![ast::Statement::ModuleCall(ast::ModuleCall {
                    function: "cube",
                    params: vec![ast::ParameterValue {
                        name: None,
                        value: ast::Expr::Vector(vec![
                            ast::Expr::Variable("a"),
                            ast::Expr::Variable("a"),
                            ast::Expr::Variable("a"),
                        ]),
                    }],
                    child: Box::new(ast::Statement::NoOp),
                })]),
            ])),
        })],
    );
}
