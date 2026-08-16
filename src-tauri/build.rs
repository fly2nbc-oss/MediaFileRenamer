fn main() {
    tauri_build::build();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("VCPKG_OVERLAY_PORTS").is_err()
    {
        println!(
            "cargo:warning=VCPKG_OVERLAY_PORTS is unset; the Windows startup-crash fix in vcpkg-overlay/ will not be applied"
        );
    }
}
