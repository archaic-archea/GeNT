use std::error::Error;
use std::path::PathBuf;
use std::env;

const LAI_SOURCES: &[&str] = &[
    "core/error.c",
    "core/eval.c",
    "core/exec.c",
    "core/exec-operand.c",
    "core/libc.c",
    "core/ns.c",
    "core/object.c",
    "core/opregion.c",
    "core/os_methods.c",
    "core/variable.c",
    "core/vsnprintf.c",
    "helpers/pc-bios.c",
    "helpers/pci.c",
    "helpers/resource.c",
    "helpers/sci.c",
    "helpers/pm.c",
    "drivers/ec.c",
    "drivers/timer.c",
];

fn main() -> Result<(), Box<dyn Error>> {
    let arch = env::var("CARGO_CFG_TARGET_ARCH")?;
    let cc = PathBuf::from(env::var("CC").unwrap_or_else(|_| "clang".into()));

    let lai_dir = String::from("lai");
    let lai_include_dir = PathBuf::from(&lai_dir).join("include");

    let mut lai_build = cc::Build::new();
    let mut lai_bindgen = bindgen::builder();

    lai_build
        .compiler(&cc)
        .archiver("llvm-ar")
        .include(&lai_include_dir)
        .flag("-fno-stack-protector")
        .flag("-fno-stack-check")
        .flag("-ffunction-sections")
        .flag("-ffreestanding")
        .pic(true)
        .files(LAI_SOURCES.iter().map(|p| format!("{lai_dir}/{p}")));

    match arch.as_str() {
        "x86_64" => {
            lai_build.target("x86_64-unknown-none");
            lai_build.flag("-mno-mmx");
            lai_build.flag("-mno-sse");
            lai_build.flag("-mno-red-zone");
            lai_bindgen = lai_bindgen.clang_arg("--target=x86_64-unknown-none");
        }
        "riscv64" => {
            lai_build.target("riscv64");
            lai_build.flag("-march=rv64imac");
            lai_build.flag("-mabi=lp64");
            lai_bindgen =
                lai_bindgen.clang_args(["--target=riscv64", "-march=rv64imac", "-mabi=lp64"]);
        }
        _ => panic!("unknown architecture."),
    }

    lai_build.compile("lai");

    // Run bindgen.

    println!("cargo:rerun-if-changed=lai-bindgen.h");
    let bindings = lai_bindgen
        .header("lai-bindgen.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args([
            &format!("-I{}", lai_include_dir.display()),
        ])
        .generate_inline_functions(true)
        .use_core()
        .generate()?;

    bindings.write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("lai.rs"))?;

    println!("cargo:rerun-if-changed=linker.lds");
    println!("cargo:rustc-link-arg=--script=linker.lds");
    println!("cargo:rustc-link-arg=-znostart-stop-gc");

    Ok(())
}