fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let version = env!("CARGO_PKG_VERSION");
    let major_version: u8 = version.split('.')
        .next()
        .unwrap()
        .parse()
        .expect("Invalid major version number");
    println!("cargo:rustc-cfg=version=\"{}\"", major_version);
}
