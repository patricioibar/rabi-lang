//! Mide el mismo programa implementado en `rabi`, Python y Rust.
//!
//! No es un test: no verifica nada, solamente imprime los tiempos por pantalla.
//! Se corre con `cargo bench`, y queda fuera de `cargo test`.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use std::{env, fs, io, process};

const RUNS: usize = 3;

fn main() {
    let bench_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/benchmark");
    let out_dir = match make_out_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("no se pudo crear el directorio temporal: {e}");
            return;
        }
    };

    let rust_bin = compile_rust(&bench_dir.join("benchmark.rs"), &out_dir);
    if let Err(e) = &rust_bin {
        eprintln!("no se pudo compilar el programa de Rust: {e}");
    }

    println!("Resultados ({RUNS} corridas por lenguaje):");

    run(
        "python3",
        Command::new("python3").arg(bench_dir.join("benchmark.py")),
    );
    run(
        "rabi",
        Command::new(env!("CARGO_BIN_EXE_rabi")).arg(bench_dir.join("benchmark.rabi")),
    );
    if let Ok(bin) = &rust_bin {
        run("rust", &mut Command::new(bin));
    }

    let _ = fs::remove_dir_all(&out_dir);
}

fn compile_rust(source: &Path, out_dir: &Path) -> Result<PathBuf, String> {
    let binary = out_dir.join("benchmark-rs");
    let status = Command::new("rustc")
        .arg("-O")
        .arg("-o")
        .arg(&binary)
        .arg(source)
        .status()
        .map_err(|e| format!("no se pudo ejecutar rustc: {e}"))?;

    if status.success() {
        Ok(binary)
    } else {
        Err(format!("rustc terminó con {status}"))
    }
}

fn run(name: &str, command: &mut Command) {
    command.stdout(Stdio::null());

    print!("{name:<10}");
    let _ = io::stdout().flush();

    let mut total = 0;
    for _ in 0..RUNS {
        match time_run(command) {
            Ok(ms) => {
                total += ms;
                print!(" {ms:>6} ms");
            }
            Err(e) => {
                println!("   {e}");
                return;
            }
        }
        let _ = io::stdout().flush();
    }

    println!("   promedio: {:>6} ms", total / RUNS as u128);
}

fn time_run(command: &mut Command) -> Result<u128, String> {
    let start = Instant::now();
    let status = command
        .status()
        .map_err(|e| format!("no se pudo ejecutar: {e}"))?;
    let elapsed = start.elapsed();

    if status.success() {
        Ok(elapsed.as_millis())
    } else {
        Err(format!("terminó con {status}"))
    }
}

fn make_out_dir() -> io::Result<PathBuf> {
    let mut dir = String::new();
    let _ = write!(dir, "rabi-benchmark-{}", process::id());
    let dir = env::temp_dir().join(dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
