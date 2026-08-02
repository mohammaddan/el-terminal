fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let local_pc = format!(
        "{manifest_dir}/.local-deps/usr/lib/x86_64-linux-gnu/pkgconfig"
    );
    let local_lib = format!("{manifest_dir}/.local-deps/usr/lib/x86_64-linux-gnu");

    if std::path::Path::new(&local_pc).exists() {
        let existing = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
        let path = if existing.is_empty() {
            local_pc.clone()
        } else {
            format!("{local_pc}:{existing}")
        };
        // SAFETY: build script runs single-threaded before rustc
        unsafe {
            std::env::set_var("PKG_CONFIG_PATH", &path);
        }
        println!("cargo:rustc-link-search=native={local_lib}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{local_lib}");
    }

    pkg_config::Config::new()
        .atleast_version("0.70")
        .probe("vte-2.91-gtk4")
        .expect(
            "vte-2.91-gtk4 not found; install libvte-2.91-gtk4-dev \
             (or extract it under .local-deps/)",
        );
}
