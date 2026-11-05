//! Mide el mismo programa implementado en `rabi`, Python y Rust, y desglosa
//! el tiempo de `rabi` por fase: scanning, parsing, optimización, compilación y
//! ejecución.
//!
//! No es un test: no verifica nada, solamente imprime los tiempos por pantalla.
//! Se corre con `cargo bench`, y queda fuera de `cargo test`.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::{env, fs, io, process};

use rabi::{compiler, interpreter::Interpreter, optimizer, parser, scanner, vm::Vm};

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

    let rabi_program = bench_dir.join("benchmark.rabi");

    println!("Proceso completo ({RUNS} corridas):");

    run(
        "python3",
        Command::new("python3").arg(bench_dir.join("benchmark.py")),
    );
    run(
        "rabi (intérprete)",
        Command::new(env!("CARGO_BIN_EXE_rabi")).arg(&rabi_program),
    );
    run(
        "rabi (vm)",
        Command::new(env!("CARGO_BIN_EXE_rabi"))
            .args(["--backend", "vm"])
            .arg(&rabi_program),
    );
    if let Ok(bin) = &rust_bin {
        run("rust", &mut Command::new(bin));
    }

    println!();
    println!("Fases de rabi, en proceso ({RUNS} corridas):");
    if let Err(e) = run_phases(&rabi_program) {
        println!("   {e}");
    }

    let _ = fs::remove_dir_all(&out_dir);
}

const PHASES: [&str; 6] = [
    "scanning",
    "parsing",
    "optimización",
    "compilación",
    "ejecución vm",
    "ejecución intérprete",
];

fn run_phases(program: &Path) -> Result<(), String> {
    let source = fs::read(program).map_err(|e| format!("no se pudo leer el programa: {e}"))?;

    let mut samples: [Vec<Duration>; PHASES.len()] = Default::default();
    for _ in 0..RUNS {
        let times = time_phases(&source)?;
        for (phase, time) in samples.iter_mut().zip(times) {
            phase.push(time);
        }
    }

    for (name, times) in PHASES.iter().zip(&samples) {
        print!("{name:<NAME_WIDTH$}");
        for time in times {
            print!(" {:>9.3} ms", millis(*time));
        }
        let average = times.iter().sum::<Duration>() / RUNS as u32;
        println!("   promedio: {:>9.3} ms", millis(average));
    }
    Ok(())
}

fn time_phases(source: &[u8]) -> Result<[Duration; PHASES.len()], String> {
    let start = Instant::now();
    let tokens = scanner::scan(source)?;
    let scanning = start.elapsed();

    let start = Instant::now();
    let statements = parser::parse(tokens)?;
    let parsing = start.elapsed();

    let for_interpreter = statements.clone();

    let start = Instant::now();
    let statements = optimizer::optimize(statements);
    let optimizing = start.elapsed();

    let start = Instant::now();
    let script = compiler::compile(statements)?;
    let compiling = start.elapsed();

    let start = Instant::now();
    Vm::with_output(io::sink()).run(script)?;
    let vm = start.elapsed();

    let start = Instant::now();
    Interpreter::with_output(io::sink()).run(&for_interpreter)?;
    let interpreter = start.elapsed();

    Ok([scanning, parsing, optimizing, compiling, vm, interpreter])
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

const NAME_WIDTH: usize = 22;

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

    print!("{name:<NAME_WIDTH$}");
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
