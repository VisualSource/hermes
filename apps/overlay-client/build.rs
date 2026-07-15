fn main() {
    #[cfg(windows)]
    {
        // nng-sys 1.0.1 uses IsValidSecurityDescriptor for Windows IPC (named
        // pipes) but doesn't declare the advapi32 dependency
        println!("cargo:rustc-link-lib=advapi32");
    }
}
