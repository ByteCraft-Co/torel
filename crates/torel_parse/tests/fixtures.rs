use torel_lexer::lex;
use torel_parse::parse_source_file;

#[test]
fn parses_minimal_unit_fixture() {
    let tokens = lex(include_str!("../../../tests/fixtures/minimal_unit.torel"));
    let file = parse_source_file(&tokens).expect("fixture should parse");

    assert_eq!(
        file.unit.expect("unit declaration").path,
        vec!["fixtures".to_owned(), "minimal".to_owned()]
    );
}
