fn main() {
    // nng-sys 1.0.1 uses IsValidSecurityDescriptor for Windows IPC (named
    // pipes) but doesn't declare the advapi32 dependency. Add it here.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
