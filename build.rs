use std::process::Command;

fn main() {
    println!("cargo:warning=NOTE:running go-pion-webrtc-rs build.rs");

    //println!("cargo:rerun-if-changed=go.mod");
    println!("cargo:rerun-if-changed=main.go");

    let out_dir = std::path::PathBuf::from(
        std::env::var("OUT_DIR").expect("failed to read env OUT_DIR"),
    );

    let mut lib_path = out_dir.clone();
    #[cfg(target_os = "macos")]
    lib_path.push("go-pion-webrtc.dylib");
    #[cfg(target_os = "windows")]
    lib_path.push("go-pion-webrtc.dll");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    lib_path.push("go-pion-webrtc.so");

    go_check_version();
    go_build(&lib_path);
    run_bindgen(&out_dir);
}

fn go_check_version() {
    println!("cargo:warning=NOTE:checking go version");

    let go_version = Command::new("go")
        .arg("version")
        .output()
        .expect("error checking go version");
    assert_eq!(b"go version go", &go_version.stdout[0..13]);
    let ver: f64 = String::from_utf8_lossy(&go_version.stdout[13..17])
        .parse()
        .expect("error parsing go version");
    assert!(
        ver >= 1.18,
        "go executable version must be >= 1.18, got: {}",
        ver
    );
}

fn go_build(path: &std::path::Path) {
    let mut cmd = Command::new("go");
    cmd.arg("build")
        .arg("-o")
        .arg(path)
        .arg("-a")
        .arg("-buildmode=c-shared");

    println!("cargo:warning=NOTE:running go build: {:?}", cmd);

    assert!(
        cmd.spawn()
            .expect("error spawing go build")
            .wait()
            .expect("error running go build")
            .success(),
        "error running go build",
    );
}

fn run_bindgen(out_dir: &std::path::PathBuf) {
    println!("cargo:warning=NOTE:running rust bindgen");

    let mut header_path = out_dir.clone();
    header_path.push("go-pion-webrtc.h");

    let mut binding_path = out_dir.clone();
    binding_path.push("go-pion-webrtc.rs");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate ffi bindings");

    bindings
        .write_to_file(binding_path)
        .expect("failed to write ffi bindings");
}
