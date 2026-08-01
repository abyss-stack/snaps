fn main() {
    if std::env::var("VERSION").is_err() {
        println!("cargo:rustc-env=VERSION=0.0.0-0");
    }
}
