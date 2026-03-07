mod utils;
use utils::parse;

#[test]
fn test_textarea_incomplete_close_then_real_close() {
    let source = "<textarea>x</textarea\n</textarea>";
    let tree = parse(source);
    println!("result: {tree}");
}
