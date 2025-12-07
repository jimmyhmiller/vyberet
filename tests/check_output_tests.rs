use pyret_attempt2::codegen::{Backend, SchemeCompiler};
use pyret_attempt2::tokenizer::Tokenizer;
use pyret_attempt2::{FileRegistry, Parser};
use std::fs;
use std::process::Command;

/// Determine which Scheme backend to use based on SCHEME_BACKEND env var
fn get_scheme_backend() -> String {
    std::env::var("SCHEME_BACKEND").unwrap_or_else(|_| "chicken".to_string())
}

/// Helper to compile a Pyret file and run it with checks enabled
fn compile_and_run_with_checks(file_path: &str) -> String {
    // Read the source file
    let source = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", file_path));

    // Get absolute path
    let absolute_path = fs::canonicalize(file_path)
        .unwrap_or_else(|_| panic!("Failed to get absolute path for: {}", file_path))
        .to_string_lossy()
        .to_string();

    // Create file registry
    let mut registry = FileRegistry::new();
    let file_id = registry.register(absolute_path);

    // Tokenize
    let mut tokenizer = Tokenizer::new(&source, file_id);
    let tokens = tokenizer.tokenize();

    // Parse
    let mut parser = Parser::new(tokens, file_id);
    let program = parser
        .parse_program()
        .unwrap_or_else(|e| panic!("Parse error in {}: {:?}", file_path, e));

    // Determine backend
    let backend_str = get_scheme_backend();
    let backend = match backend_str.as_str() {
        "ribbit" => Backend::Ribbit,
        "gambit" => Backend::Gambit,
        "chez" => Backend::Chez,
        _ => Backend::Chicken,
    };

    // Compile with checks enabled
    let mut compiler = SchemeCompiler::new();
    compiler.set_enable_checks(true);
    compiler.set_file_registry(registry);
    compiler.set_backend(backend);
    let scheme_code = compiler
        .compile_program(&program)
        .unwrap_or_else(|e| panic!("Compilation error in {}: {}", file_path, e));

    // Write to temp file with unique name
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_pyret_{}_{}.scm",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Read runtime
    let runtime_code =
        fs::read_to_string("runtime/runtime.scm").expect("Failed to read runtime library");

    // For Ribbit, also load rationals library
    let runtime_with_rationals = if backend_str == "ribbit" {
        let rationals_code = fs::read_to_string("runtime/ribbit-additions.scm")
            .expect("Failed to read rationals library");
        format!("{}\n\n{}", runtime_code, rationals_code)
    } else {
        runtime_code
    };

    let full_code = format!("{}\n\n{}", runtime_with_rationals, scheme_code);
    fs::write(&temp_file, full_code).expect("Failed to write temp file");

    // Determine backend and run
    let backend = get_scheme_backend();
    let output = match backend.as_str() {
        "chicken" => {
            // Run with Chicken Scheme
            Command::new("csi")
                .arg("-q")
                .arg("-b")
                .arg(&temp_file)
                .output()
                .expect("Failed to run csi")
        }
        "gambit" => {
            // Run with Gambit Scheme
            Command::new("gsi")
                .arg(&temp_file)
                .output()
                .expect("Failed to run gsi")
        }
        "chez" => {
            // Run with Chez Scheme
            Command::new("chez")
                .arg("--script")
                .arg(&temp_file)
                .output()
                .expect("Failed to run chez")
        }
        "ribbit" => {
            // Compile with Ribbit to C, then run
            let ribbit_dir = std::env::var("RIBBIT_DIR")
                .map(|d| format!("{}/src", d))
                .unwrap_or_else(|_| {
                    "/Users/jimmyhmiller/Documents/Code/open-source/ribbit/src".to_string()
                });

            // Use a single timestamp for both filenames to avoid race conditions
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let c_file =
                temp_dir.join(format!("test_pyret_{}_{}.c", std::process::id(), timestamp));
            let exe_file =
                temp_dir.join(format!("test_pyret_{}_{}", std::process::id(), timestamp));

            // Compile Scheme to C with Ribbit
            let rsc_output = Command::new(format!("{}/rsc.exe", ribbit_dir))
                .arg("-t")
                .arg("c")
                .arg("-l")
                .arg("r4rs")
                .arg(&temp_file)
                .arg("-o")
                .arg(&c_file)
                .output()
                .expect("Failed to run Ribbit compiler");

            if !rsc_output.status.success() {
                panic!(
                    "Ribbit compilation failed: {}",
                    String::from_utf8_lossy(&rsc_output.stderr)
                );
            }

            // Verify C file was created
            if !c_file.exists() {
                panic!(
                    "Ribbit compilation succeeded but C file not created at {:?}\nstdout: {}\nstderr: {}",
                    c_file,
                    String::from_utf8_lossy(&rsc_output.stdout),
                    String::from_utf8_lossy(&rsc_output.stderr)
                );
            }

            // Compile C to executable
            let gcc_output = Command::new("gcc")
                .arg(&c_file)
                .arg("-o")
                .arg(&exe_file)
                .output()
                .expect("Failed to run gcc");

            if !gcc_output.status.success() {
                panic!(
                    "GCC compilation failed: {}",
                    String::from_utf8_lossy(&gcc_output.stderr)
                );
            }

            // Run executable
            let exe_output = Command::new(&exe_file)
                .output()
                .expect("Failed to run Ribbit executable");

            // Clean up intermediate files
            let _ = fs::remove_file(&c_file);
            let _ = fs::remove_file(&exe_file);

            exe_output
        }
        _ => panic!(
            "Unknown SCHEME_BACKEND: {}. Use 'chicken', 'gambit', 'chez', or 'ribbit'",
            backend
        ),
    };

    // Clean up
    let _ = fs::remove_file(&temp_file);

    // Return stdout
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to run Pyret and get its output
fn run_pyret(file_path: &str) -> String {
    let output = Command::new("pyret")
        .arg(file_path)
        .output()
        .expect("Failed to run pyret");

    String::from_utf8_lossy(&output.stdout).to_string()
}

// Tests that compare against Pyret output - require Pyret to be installed
// Run with: cargo test -- --ignored

#[test]
#[ignore = "requires Pyret CLI"]
fn test_checks_simple() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/checks.arr");
    let pyret_output = run_pyret("tests/pyret-files/checks.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for checks.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_checks_multiple_blocks() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/check_operators.arr");
    let pyret_output = run_pyret("tests/pyret-files/check_operators.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for check_operators.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_no_checks() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/factorial.arr");
    let pyret_output = run_pyret("tests/pyret-files/factorial.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for factorial.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_fibonacci_no_checks() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/fibonacci.arr");
    let pyret_output = run_pyret("tests/pyret-files/fibonacci.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for fibonacci.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_arithmetic_no_checks() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/arithmetic.arr");
    let pyret_output = run_pyret("tests/pyret-files/arithmetic.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for arithmetic.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_fractions() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/fractions.arr");
    let pyret_output = run_pyret("tests/pyret-files/fractions.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for fractions.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_numeric_functions() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/numeric_functions.arr");
    let pyret_output = run_pyret("tests/pyret-files/numeric_functions.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for numeric_functions.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_lambdas() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_lambdas.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_lambdas.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_lambdas.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_strings() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_strings.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_strings.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_strings.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_blocks() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_blocks.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_blocks.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_blocks.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_mutable_vars() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_mutable_vars.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_mutable_vars.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_mutable_vars.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_conditionals() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_conditionals.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_conditionals.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_conditionals.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_functions() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_functions.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_functions.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_functions.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_tuples() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_tuples.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_tuples.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_tuples.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_lists() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_lists.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_lists.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_lists.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

#[test]
#[ignore = "requires Pyret CLI"]
fn test_binary_operators() {
    let our_output = compile_and_run_with_checks("tests/pyret-files/test_binary_operators.arr");
    let pyret_output = run_pyret("tests/pyret-files/test_binary_operators.arr");

    assert_eq!(
        our_output, pyret_output,
        "Output mismatch for test_binary_operators.arr:\nOurs:\n{}\n\nPyret:\n{}",
        our_output, pyret_output
    );
}

// Tests that don't require Pyret - they verify expected output patterns

#[test]
fn test_cases_simple() {
    let output = compile_and_run_with_checks("tests/pyret-files/cases_simple.arr");

    // Verify all tests passed
    assert!(
        output.contains("Looks shipshape, all 2 tests passed, mate!"),
        "Expected 2 tests to pass in cases_simple.arr. Output:\n{}",
        output
    );
}

#[test]
fn test_cases_comprehensive() {
    let output = compile_and_run_with_checks("tests/pyret-files/cases_comprehensive.arr");

    // Verify all tests passed
    assert!(
        output.contains("Looks shipshape, all 8 tests passed, mate!"),
        "Expected 8 tests to pass in cases_comprehensive.arr. Output:\n{}",
        output
    );
}

#[test]
fn test_for_loops() {
    let output = compile_and_run_with_checks("tests/pyret-files/for_loops.arr");

    // Verify all tests passed
    assert!(
        output.contains("Looks shipshape, all 4 tests passed, mate!"),
        "Expected 4 tests to pass in for_loops.arr. Output:\n{}",
        output
    );

    // Verify output includes expected results
    assert!(
        output.contains("[list: 2, 4, 6, 8, 10]"),
        "Expected doubled list"
    );
    assert!(output.contains("[list: 2, 4]"), "Expected filtered evens");
    assert!(output.contains("15"), "Expected sum of 15");
}

#[test]
fn test_for_map_simple() {
    let output = compile_and_run_with_checks("tests/pyret-files/for_map_simple.arr");

    // Verify map output
    assert!(
        output.contains("[list: 2, 4, 6]"),
        "Expected [list: 2, 4, 6]. Output:\n{}",
        output
    );
}

#[test]
fn test_objects_with_checks() {
    let output = compile_and_run_with_checks("tests/pyret-files/objects_with_checks.arr");

    // Verify all tests passed
    assert!(
        output.contains("Looks shipshape, all 10 tests passed, mate!"),
        "Expected 10 tests to pass in objects_with_checks.arr. Output:\n{}",
        output
    );
}

#[test]
fn test_method_calls() {
    let output = compile_and_run_with_checks("tests/pyret-files/test_method_calls.arr");

    // Verify all tests passed (3 check blocks with multiple tests)
    assert!(
        output.contains("Looks shipshape"),
        "Expected all tests to pass in test_method_calls.arr. Output:\n{}",
        output
    );
}
