use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR")?);

    // workspace では memory.x へのパスが曖昧なので、明示的に
    // OUT_DIR に書き出してディレクトリをリンカーに渡す。
    let memory_x = out.join("memory.x");
    File::create(&memory_x)?.write_all(include_bytes!("memory.x"))?;

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    // memory.x の変更にだけ応じて build.rs を再実行
    println!("cargo:rerun-if-changed=memory.x");

    Ok(())
}
