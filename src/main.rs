mod ast;
mod regex;

// fn parse(pattern: &str) -> Result<ast::AST> {
//
// }

#[allow(dead_code)]
enum Type {
    ERE,
    BRE,
    PCRE,
}

fn main() {
    println!("Hello, world!");

    let pattern = "a{1,2}b[ac-z]*";
    let ast = ast::Parser::new().parse(pattern);

    match ast {
        Ok(ast) => println!("{:#?}", ast),
        Err(err) => println!("{}", err),
    }
}
