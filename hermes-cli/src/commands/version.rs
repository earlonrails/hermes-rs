pub fn run_version() {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    println!("Hermes Agent v{} (Rust rewrite)", version);
    if !authors.is_empty() {
        println!("Authors: {}", authors);
    }
    println!("Inspired by the original Python Hermes Agent from Nous Research.");
}
