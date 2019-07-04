use loa;
use loa::format::Format;

fn main() -> std::io::Result<()> {
    let source = loa::Source::stdin()?;
    let expression = loa::syntax::Parser::new(&source).parse_expression();
    expression.map(|exp| {
        println!("{}", &exp as &Format);
    });
    Ok(())
}
