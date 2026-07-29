fn main() {
    if std::env::var("PROJECT_VERSION").is_err() {
        println!("cargo:rustc-env=PROJECT_VERSION=0.0.0-0");
    }
}
