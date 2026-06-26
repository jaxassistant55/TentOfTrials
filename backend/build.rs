fn main() {
    println!("cargo:rerun-if-changed=../frailbox/connector/api.c");
    println!("cargo:rerun-if-changed=../frailbox/connector/api.h");
    println!("cargo:rerun-if-changed=../frailbox/connector/protocol.h");

    #[cfg(unix)]
    {
        cc::Build::new()
            .file("../frailbox/connector/api.c")
            .include("../frailbox/connector")
            .warnings(false)
            .compile("tent_connector");
        println!("cargo:rustc-link-lib=pthread");
    }
}
