fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    let out_src = out_dir.join("srand.c");
    std::fs::write(&out_src, "__attribute__((weak)) void srand(unsigned) {}").unwrap();
    cc::Build::new().file(&out_src).compile("srand");
}
