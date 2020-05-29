pub mod ast;
//lalrpop_mod!(rscad);
mod rscad;

mod interpreter;
mod parser;

/// Parse an OpenSCAD document and outputs the AST.
pub fn parse<'a>(content: &'a str) -> Result<Vec<ast::Statement<'a>>, impl std::error::Error + 'a> {
    rscad::DocumentParser::new().parse(content)
}
