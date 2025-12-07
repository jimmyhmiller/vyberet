use pyret_attempt2::codegen::{Backend, SchemeCompiler};
use pyret_attempt2::module_compiler::ModuleCompiler;
use pyret_attempt2::tokenizer::Tokenizer;
use pyret_attempt2::{FileRegistry, Parser};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command, Stdio};

#[derive(Debug)]
struct Options {
    input_file: String,
    output_file: Option<String>,
    run: bool,
    interpreter: String,
    show_scheme: bool,
    enable_checks: bool,
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = env::args().collect();

    // Check for help first
    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        return Err(format!(
            "Usage: {} <input-file> [options]\n\
             \n\
             Options:\n\
             --output <file>            Write Scheme output to file (default: stdout)\n\
             --run                      Execute the compiled Scheme code\n\
             --interpreter <name>       Specify interpreter: chicken, gsi, ribbit (default: chicken)\n\
             --show-scheme              Show generated Scheme code before running\n\
             --check                    Enable check blocks (default: disabled)\n\
             --help                     Show this help message\n\
             \n\
             Short forms: -o, -r, -i, -s, -c, -h\n\
             \n\
             Examples:\n\
             {} input.arr                                  # Compile to stdout\n\
             {} input.arr --output output.scm              # Compile to file\n\
             {} input.arr --run                            # Compile and run with Chicken\n\
             {} input.arr --run --interpreter gsi          # Compile and run with Gambit\n\
             {} input.arr --run --show-scheme              # Compile, show code, and run\n\
             {} input.arr --run --interpreter ribbit       # Compile to native binary and run",
            args[0], args[0], args[0], args[0], args[0], args[0], args[0]
        ));
    }

    let mut opts = Options {
        input_file: args[1].clone(),
        output_file: None,
        run: false,
        interpreter: "chicken".to_string(),
        show_scheme: false,
        enable_checks: false,
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    return Err("Missing argument for -o/--output".to_string());
                }
                opts.output_file = Some(args[i + 1].clone());
                i += 2;
            }
            "-r" | "--run" => {
                opts.run = true;
                i += 1;
            }
            "-i" | "--interpreter" => {
                if i + 1 >= args.len() {
                    return Err("Missing argument for -i/--interpreter".to_string());
                }
                opts.interpreter = args[i + 1].clone();
                if !["chicken", "csi", "gsi", "ribbit"].contains(&opts.interpreter.as_str()) {
                    return Err(format!(
                        "Unknown interpreter: {}. Supported: chicken, gsi, ribbit",
                        opts.interpreter
                    ));
                }
                i += 2;
            }
            "-s" | "--show-scheme" => {
                opts.show_scheme = true;
                i += 1;
            }
            "-c" | "--check" => {
                opts.enable_checks = true;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    Ok(opts)
}

fn main() {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let input_file = &opts.input_file;
    let input_path = Path::new(input_file);

    // Detect project root from input file
    let project_root = ModuleCompiler::detect_project_root(input_path);

    // Use the multi-file module compiler with detected project root
    let mut module_compiler = ModuleCompiler::new(project_root);
    let scheme_code = match module_compiler.compile_program(input_path) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            std::process::exit(1);
        }
    };

    // Show scheme code if requested
    if opts.show_scheme && opts.run {
        eprintln!("==== Generated Scheme Code ====");
        eprintln!("{}", scheme_code);
        eprintln!("==== Execution Output ====");
    }

    // Write output if specified
    if let Some(ref out_path) = opts.output_file {
        if let Err(e) = fs::write(out_path, &scheme_code) {
            eprintln!("Error writing output file '{}': {}", out_path, e);
            std::process::exit(1);
        }
        if !opts.run {
            eprintln!("Compiled successfully to: {}", out_path);
        }
    } else if !opts.run {
        // Write to stdout if not running
        print!("{}", scheme_code);
    }

    // Execute if requested
    if opts.run {
        if let Err(e) = run_scheme(&scheme_code, &opts.interpreter) {
            eprintln!("Execution error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_scheme(scheme_code: &str, interpreter: &str) -> Result<(), String> {
    // Create a temporary file with runtime + code
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("pyret_temp_{}.scm", std::process::id()));

    // Check if the scheme code already includes the runtime library
    // (ModuleCompiler includes it automatically)
    let full_code = if scheme_code.contains("; ===== Runtime Library =====") {
        // Runtime already included, use as-is
        scheme_code.to_string()
    } else {
        // Need to prepend runtime library
        // Read runtime library
        let runtime_path = "runtime/runtime.scm";
        let runtime_code = fs::read_to_string(runtime_path)
            .map_err(|e| format!("Failed to read runtime library at {}: {}", runtime_path, e))?;

        // For Ribbit, also load the rationals library
        let runtime_with_rationals = if interpreter == "ribbit" {
            let rationals_path = "runtime/rationals.scm";
            let rationals_code = fs::read_to_string(rationals_path)
                .map_err(|e| format!("Failed to read rationals library at {}: {}", rationals_path, e))?;
            format!("{}\n\n; ==== Rational Number Support (Ribbit) ====\n{}", runtime_code, rationals_code)
        } else {
            runtime_code
        };

        // Combine runtime and code
        format!("{}\n\n; ==== Compiled Pyret Code ====\n{}", runtime_with_rationals, scheme_code)
    };

    // Write to temp file
    fs::write(&temp_file, full_code)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Run with appropriate interpreter
    let result = match interpreter {
        "chicken" | "csi" => {
            Command::new("csi")
                .arg("-q")
                .arg("-b")
                .arg(&temp_file)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
        }
        "gsi" => {
            Command::new("gsi")
                .arg("-:d-")
                .arg(&temp_file)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
        }
        "ribbit" => {
            return run_ribbit(&temp_file);
        }
        _ => {
            return Err(format!("Unknown interpreter: {}", interpreter));
        }
    };

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    match result {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("Interpreter exited with status: {}", status))
            }
        }
        Err(e) => Err(format!("Failed to execute {}: {}", interpreter, e)),
    }
}

fn run_ribbit(scheme_file: &std::path::Path) -> Result<(), String> {
    // Find Ribbit directory from environment or use default
    let ribbit_dir_str = std::env::var("RIBBIT_DIR")
        .unwrap_or_else(|_| "/Users/jimmyhmiller/Documents/Code/open-source/ribbit".to_string());
    let ribbit_dir = std::path::Path::new(&ribbit_dir_str);
    let rsc_exe = ribbit_dir.join("src/rsc.exe");

    // Check if rsc.exe exists
    if !rsc_exe.exists() {
        return Err(format!(
            "Ribbit compiler not found at {:?}. Run 'cd {} && make rsc.exe' first.",
            rsc_exe,
            ribbit_dir.join("src").display()
        ));
    }

    // Create temp files for C output and executable
    let temp_dir = std::env::temp_dir();
    let c_file = temp_dir.join(format!("pyret_ribbit_{}.c", std::process::id()));
    let exe_file = temp_dir.join(format!("pyret_ribbit_{}", std::process::id()));

    // Compile Scheme to C using Ribbit
    // Use r4rs library instead of min for full R4RS support (including I/O)
    let compile_status = Command::new(&rsc_exe)
        .arg("-t")
        .arg("c")
        .arg("-l")
        .arg("r4rs")
        .arg(scheme_file)
        .arg("-o")
        .arg(&c_file)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run Ribbit compiler: {}", e))?;

    if !compile_status.success() {
        let _ = fs::remove_file(&c_file);
        return Err("Ribbit compilation failed".to_string());
    }

    // Compile C to executable
    let gcc_status = Command::new("gcc")
        .arg(&c_file)
        .arg("-o")
        .arg(&exe_file)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to compile C code: {}", e))?;

    if !gcc_status.success() {
        let _ = fs::remove_file(&c_file);
        let _ = fs::remove_file(&exe_file);
        return Err("C compilation failed".to_string());
    }

    // Run the executable
    let run_status = Command::new(&exe_file)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to execute compiled binary: {}", e))?;

    // Clean up
    let _ = fs::remove_file(&c_file);
    let _ = fs::remove_file(&exe_file);

    if run_status.success() {
        Ok(())
    } else {
        Err(format!("Program exited with status: {}", run_status))
    }
}
