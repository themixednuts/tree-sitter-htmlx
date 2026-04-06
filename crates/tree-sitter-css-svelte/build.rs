fn main() {
    println!("cargo:rerun-if-changed=src/parser.c");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-env-changed=TREE_SITTER_CSS_PROFILE");

    let mut build = cc::Build::new();
    build
        .std("c11")
        .include("src")
        .file("src/parser.c")
        .file("src/scanner.c")
        .warnings(false);

    if std::env::var_os("TREE_SITTER_CSS_PROFILE").is_some() {
        build.define("TREE_SITTER_CSS_PROFILE", None);
    }

    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");

    build.compile("tree_sitter_css");
}
