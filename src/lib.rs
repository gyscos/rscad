#[macro_use]
extern crate lalrpop_util;

lalrpop_util::lalrpop_mod!(rscad);

pub mod ast;
mod interpreter;
mod parser;

/// Parse an OpenSCAD document and outputs the AST.
pub fn parse<'a>(content: &'a str) -> Result<Vec<ast::Statement<'a>>, impl std::error::Error + 'a> {
    rscad::DocumentParser::new().parse(content)
}
