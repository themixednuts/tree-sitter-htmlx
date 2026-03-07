mod utils;
use utils::parse;

#[test]
fn test_textarea_incomplete_close_then_real_close() {
    let source = "<textarea>x</textarea\n</textarea>";
    let tree = parse(source);
    assert!(!tree.is_empty(), "Should parse without crash: {tree}");
}

#[test]
fn test_textarea_complex_malformed() {
    let source = "<textarea>\n\t<p>not actu </textar ally an element. {foo}</p>\n</textare\n\n\n> </textaread >asdf</textarea\n\n\n</textarea\n\n>\n";
    let tree = parse(source);
    assert!(!tree.is_empty(), "Should parse without crash: {tree}");
}
