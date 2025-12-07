//! Pyret to R4RS Scheme compiler
//!
//! Compiles Pyret AST nodes to R4RS Scheme code.
//! Handles name mangling, runtime support, and encoding of complex features.
//!
//! Phase 1: Core expressions (numbers, booleans, strings, operators, identifiers)
//! Phase 2: Functions, lambdas, bindings, closures
//! Phase 3+: Control flow, data structures, pattern matching, etc.

use crate::ast::{BinOp, CheckOp, Expr, FileRegistry, Import, ImportType, Name, Program, Provide};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Scheme backend target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Chicken, // Chicken Scheme (default)
    Gambit,  // Gambit Scheme
    Chez,    // Chez Scheme
    Ribbit,  // Ribbit Scheme (requires custom rational support)
}

impl Default for Backend {
    fn default() -> Self {
        Backend::Chicken
    }
}

/// Information about a variant's fields
#[derive(Debug, Clone)]
struct VariantInfo {
    fields: Vec<String>, // Field names in order
}

/// Information about a data type's methods
#[derive(Debug, Clone)]
struct DataTypeInfo {
    #[allow(dead_code)]
    variants: Vec<String>, // Names of all variants for this data type
    methods: HashSet<String>, // Names of all methods defined on this data type
}

/// Module identity - uniquely identifies a module by its URI
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ModuleUri(String);

impl ModuleUri {
    /// Create a module URI from a file path, relative to project root
    fn from_file_path(path: &Path, project_root: &Path) -> Result<Self, String> {
        let absolute = path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path {:?}: {}", path, e))?;

        // Make path relative to project root
        let relative = absolute.strip_prefix(project_root).map_err(|_| {
            format!(
                "Path {:?} is not within project root {:?}",
                absolute, project_root
            )
        })?;

        // Convert to forward slashes for consistency
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        let uri = format!("file://{}", relative_str);
        Ok(ModuleUri(uri))
    }

    /// Create a builtin module URI
    fn builtin(name: &str) -> Self {
        ModuleUri(format!("builtin://{}", name))
    }

    /// Get the URI string
    fn as_str(&self) -> &str {
        &self.0
    }

    /// Compute SHA256 hash of the URI
    fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Get the first N characters of the hash (for readable function names)
    fn hash_prefix(&self, len: usize) -> String {
        let hash = self.hash();
        hash.chars().take(len).collect()
    }

    /// Get the basename from the URI (for readable names)
    fn basename(&self) -> String {
        if self.0.starts_with("builtin://") {
            self.0.strip_prefix("builtin://").unwrap().to_string()
        } else if self.0.starts_with("file://") {
            Path::new(&self.0.strip_prefix("file://").unwrap())
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("module")
                .to_string()
        } else {
            "module".to_string()
        }
    }
}

/// Module information tracked during compilation
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ModuleInfo {
    uri: ModuleUri,
    /// What names this module provides/exports
    provides: HashSet<String>,
}

pub struct SchemeCompiler {
    indent_level: usize,
    // Track which variables are mutable (created with 'var')
    mutable_vars: HashSet<String>,
    // Whether to compile check blocks
    enable_checks: bool,
    // File registry for looking up filenames
    file_registry: Option<FileRegistry>,
    // Backend target (determines code generation strategy)
    backend: Backend,
    // Track data variant field information
    // Map from variant name to its field information
    variant_fields: HashMap<String, VariantInfo>,
    // Track data type information
    // Map from data type name to its information (variants and methods)
    data_types: HashMap<String, DataTypeInfo>,
    // Map from variant name to its data type name
    variant_to_datatype: HashMap<String, String>,
    // Current module URI being compiled (for namespacing functions)
    current_module_uri: Option<ModuleUri>,
    // Current module's source file path (for resolving relative imports)
    current_module_path: Option<PathBuf>,
    // Project root directory (for project-relative URIs)
    project_root: PathBuf,
    // Track imported modules: import alias -> module URI
    imports: HashMap<String, ModuleUri>,
    // Track module information: module URI -> module info
    modules: HashMap<ModuleUri, ModuleInfo>,
    // Track singleton constructors (zero-argument data constructors)
    // These can be used as values without calling them
    singleton_constructors: HashSet<String>,
    // Track top-level function names (user-defined functions)
    // These need namespacing when called
    toplevel_functions: HashSet<String>,
    // Track builtin data types that cannot be shadowed
    builtin_types: HashSet<String>,
}

impl SchemeCompiler {
    pub fn new() -> Self {
        Self::with_project_root(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn with_project_root(project_root: PathBuf) -> Self {
        // Initialize builtin types that cannot be shadowed
        let mut builtin_types = HashSet::new();
        builtin_types.insert("Option".to_string());

        SchemeCompiler {
            indent_level: 0,
            mutable_vars: HashSet::new(),
            enable_checks: false,
            file_registry: None,
            backend: Backend::default(),
            variant_fields: HashMap::new(),
            data_types: HashMap::new(),
            variant_to_datatype: HashMap::new(),
            current_module_uri: None,
            current_module_path: None,
            project_root,
            imports: HashMap::new(),
            modules: HashMap::new(),
            singleton_constructors: HashSet::new(),
            toplevel_functions: HashSet::new(),
            builtin_types,
        }
    }

    pub fn set_enable_checks(&mut self, enable: bool) {
        self.enable_checks = enable;
    }

    pub fn set_file_registry(&mut self, registry: FileRegistry) {
        self.file_registry = Some(registry);
    }

    pub fn set_backend(&mut self, backend: Backend) {
        self.backend = backend;
    }

    pub fn get_backend(&self) -> Backend {
        self.backend
    }

    /// Set the current module by file path
    pub fn set_module_from_path(&mut self, filepath: &Path) -> Result<(), String> {
        let uri = ModuleUri::from_file_path(filepath, &self.project_root)?;
        self.current_module_uri = Some(uri.clone());
        self.current_module_path = Some(filepath.to_path_buf());

        // Register this module
        self.modules.insert(
            uri.clone(),
            ModuleInfo {
                uri,
                provides: HashSet::new(),
            },
        );

        Ok(())
    }

    /// Set the current module for a builtin
    pub fn set_builtin_module(&mut self, name: &str) {
        let uri = ModuleUri::builtin(name);
        self.current_module_uri = Some(uri.clone());

        self.modules.insert(
            uri.clone(),
            ModuleInfo {
                uri,
                provides: HashSet::new(),
            },
        );
    }

    /// Legacy method for compatibility
    #[deprecated(note = "Use set_module_from_path instead")]
    pub fn set_module_name(&mut self, module_name: String) {
        // Convert to builtin module for now
        self.set_builtin_module(&module_name);
    }

    /// Derive module name from a file path
    /// Examples: "tests/pyret-files/trove/lists.arr" -> "lists"
    ///           "simple.arr" -> "simple"
    pub fn derive_module_name(filepath: &str) -> String {
        std::path::Path::new(filepath)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("main")
            .to_string()
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    /// Mangle a Pyret identifier to valid Scheme identifier
    /// Handles special characters that may appear in Pyret names
    fn mangle_name(&self, name: &str) -> String {
        name.chars()
            .flat_map(|c| match c {
                '-' => vec!['_', 'd', 'a', 's', 'h', '_'],
                '?' => vec!['_', 'p', '_'],
                '!' => vec!['_', 'b', 'a', 'n', 'g', '_'],
                '<' => vec!['_', 'l', 't', '_'],
                '>' => vec!['_', 'g', 't', '_'],
                '=' => vec!['_', 'e', 'q', '_'],
                '+' => vec!['_', 'p', 'l', 'u', 's', '_'],
                '*' => vec!['_', 's', 't', 'a', 'r', '_'],
                '/' => vec!['_', 's', 'l', 'a', 's', 'h', '_'],
                '%' => vec!['_', 'p', 'e', 'r', 'c', 'e', 'n', 't', '_'],
                '^' => vec!['_', 'c', 'a', 'r', 'e', 't', '_'],
                '&' => vec!['_', 'a', 'm', 'p', '_'],
                '|' => vec!['_', 'p', 'i', 'p', 'e', '_'],
                c if c.is_alphanumeric() || c == '_' => vec![c],
                _ => vec!['_', 'x', '_'], // fallback for other chars
            })
            .collect()
    }

    /// Namespace a function name with the current module using hash-based prefix
    /// Examples:
    ///   "foo" in file:///proj-a/util.arr -> "util_a7b3c2d1__foo"
    ///   "bar" in builtin://lists -> "lists_builtin__bar"
    fn namespace_function_name(&self, name: &str) -> String {
        if let Some(ref uri) = self.current_module_uri {
            let basename = uri.basename();
            let hash = if uri.as_str().starts_with("builtin://") {
                // For built-ins, use "builtin" as the hash suffix for readability
                "builtin".to_string()
            } else {
                // For file modules, use 8-character hash prefix
                uri.hash_prefix(8)
            };
            format!(
                "{}_{}__{}",
                self.mangle_name(&basename),
                hash,
                self.mangle_name(name)
            )
        } else {
            // Fallback if no module URI set (shouldn't happen in normal use)
            self.mangle_name(name)
        }
    }

    /// Get the function prefix for a module URI (without the function name)
    /// Used for resolving imported functions
    fn get_module_prefix(&self, uri: &ModuleUri) -> String {
        let basename = uri.basename();
        let hash = if uri.as_str().starts_with("builtin://") {
            "builtin".to_string()
        } else {
            uri.hash_prefix(8)
        };
        format!("{}_{}", self.mangle_name(&basename), hash)
    }

    /// Process import statements and register them
    fn process_imports(&mut self, imports: &[Import]) -> Result<(), String> {
        use crate::ast::{Import, Name};

        for import in imports {
            match import {
                Import::SImport { import, name, .. } => {
                    // Get the alias name
                    let alias = match name {
                        Name::SName { s, .. } => s.clone(),
                        _ => return Err("Import alias must be a simple name".to_string()),
                    };

                    // Resolve the import source to a URI
                    let uri = self.resolve_import_source(import)?;

                    // Register the import
                    self.imports.insert(alias, uri.clone());

                    // Register the module if not already present
                    if !self.modules.contains_key(&uri) {
                        self.modules.insert(
                            uri.clone(),
                            ModuleInfo {
                                uri,
                                provides: HashSet::new(),
                            },
                        );
                    }
                }
                Import::SInclude { import, .. } => {
                    // Include brings names directly into scope (no prefix needed)
                    // For now, we'll just resolve it to track dependencies
                    let _uri = self.resolve_import_source(import)?;
                    // TODO: Track included names for direct access
                }
                _ => {
                    // Other import types not yet supported
                    // (SIncludeFrom, SImportFields, SImportTypes)
                }
            }
        }

        Ok(())
    }

    /// Resolve an import source to a module URI
    fn resolve_import_source(&self, import: &ImportType) -> Result<ModuleUri, String> {
        match import {
            ImportType::SConstImport { module, .. } => {
                // Builtin module (no prefix like "file(...)")
                Ok(ModuleUri::builtin(module))
            }
            ImportType::SSpecialImport { kind, args, .. } => {
                match kind.as_str() {
                    "file" if !args.is_empty() => {
                        // File import - resolve relative to current module's directory
                        let file_path = &args[0];
                        let import_path = Path::new(file_path);

                        // Resolve relative to current module's directory
                        let resolved_path = if import_path.is_absolute() {
                            // Absolute path - use as-is
                            import_path.to_path_buf()
                        } else if let Some(ref current_path) = self.current_module_path {
                            // Relative path - resolve relative to current module's directory
                            if let Some(parent) = current_path.parent() {
                                parent.join(import_path)
                            } else {
                                import_path.to_path_buf()
                            }
                        } else {
                            // No current module path - use relative to current directory
                            import_path.to_path_buf()
                        };

                        ModuleUri::from_file_path(&resolved_path, &self.project_root)
                    }
                    _ => Err(format!("Unsupported import kind: {}", kind)),
                }
            }
        }
    }

    /// Process provide statement to track exported names
    fn process_provides(&mut self, provide: &Provide) -> Result<(), String> {
        match provide {
            Provide::SProvideAll { .. } => {
                // Provide everything - mark all top-level functions as provided
                // This will be populated as we compile the program
                Ok(())
            }
            Provide::SProvideNone { .. } => {
                // Provide nothing
                Ok(())
            }
            Provide::SProvide { .. } => {
                // Specific provides - would need to parse the block
                // For now, treat as provide-all
                Ok(())
            }
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        // Add a header comment
        let mut output = String::from("; Generated from Pyret by vyberet compiler\n");
        output.push_str("; Target: R4RS Scheme\n\n");

        // Process imports
        self.process_imports(&program.imports)?;

        // Process provides to track what this module exports
        self.process_provides(&program._provide)?;

        // Compile the program body
        // If it's a block with definitions, we need to extract them
        let body_code = match &*program.body {
            Expr::SBlock { stmts, .. } if !stmts.is_empty() => {
                // Extract definitions and final expression
                for stmt in stmts.iter() {
                    let stmt_code = self.compile_expr(stmt)?;
                    output.push_str(&stmt_code);
                    output.push('\n');
                }
                // Add check summary at the end if checks are enabled
                if self.enable_checks {
                    output.push_str("\n(pyret:check-block-end)\n");
                }
                return Ok(output);
            }
            _ => {
                // Single expression - compile and wrap
                self.compile_expr(&program.body)?
            }
        };

        // Just output the body code - Pyret doesn't auto-print
        output.push_str(&body_code);
        output.push('\n');

        // Add check summary at the end if checks are enabled
        if self.enable_checks {
            output.push_str("\n(pyret:check-block-end)\n");
        }

        Ok(output)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            // ===== Literals =====
            Expr::SNum { value, .. } => Ok(value.clone()),

            Expr::SBool { b, .. } => Ok(if *b { "#t" } else { "#f" }.to_string()),

            Expr::SStr { s, .. } => {
                // Escape string for Scheme
                let mut escaped = String::from("\"");
                for ch in s.chars() {
                    match ch {
                        '"' => escaped.push_str("\\\""),
                        '\\' => escaped.push_str("\\\\"),
                        '\n' => escaped.push_str("\\n"),
                        '\r' => escaped.push_str("\\r"),
                        '\t' => escaped.push_str("\\t"),
                        c => escaped.push(c),
                    }
                }
                escaped.push('"');
                Ok(escaped)
            }

            Expr::SFrac { num, den, .. } => {
                // Exact rational: num/den
                if self.backend == Backend::Ribbit {
                    Ok(format!("(make-rat {} {})", num, den))
                } else {
                    Ok(format!("(/ {} {})", num, den))
                }
            }

            Expr::SRfrac { num, den, .. } => {
                // Rough/inexact rational
                if self.backend == Backend::Ribbit {
                    // Ribbit doesn't have floats, so we represent as exact rational
                    Ok(format!("(make-rat {} {})", num, den))
                } else {
                    // Convert to inexact
                    Ok(format!("(exact->inexact (/ {} {}))", num, den))
                }
            }

            // ===== Identifiers =====
            Expr::SId { id, .. } => {
                let var_name = self.compile_name(id);

                // Extract the raw name for checking singleton constructors
                let raw_name = match id {
                    Name::SName { s, .. } => s.clone(),
                    _ => String::new(),
                };

                // Check if this is a singleton constructor used as a value
                if !raw_name.is_empty() && self.singleton_constructors.contains(&raw_name) {
                    // Return the value, not the function
                    let constructor_name = self.namespace_function_name(&raw_name);
                    return Ok(format!("{}-value", constructor_name));
                }

                // Check if this is a reference to a mutable variable
                if self.mutable_vars.contains(&var_name) {
                    // It's a var - need to unbox it
                    Ok(format!("(box-ref {})", var_name))
                } else {
                    // Regular immutable binding
                    Ok(var_name)
                }
            }

            Expr::SIdVar { id, .. } => {
                // Var reference - need to unbox it
                // var x = 5 creates a box, x in an expression gets the value
                let var_name = self.compile_name(id);
                Ok(format!("(box-ref {})", var_name))
            }

            Expr::SOp {
                op, left, right, ..
            } => {
                // For Ribbit backend, use custom rational operators
                // For other backends, use native Scheme operators
                let op_str = if self.backend == Backend::Ribbit {
                    match op {
                        BinOp::Plus => "pyret:+", // Use polymorphic + for strings and rationals
                        BinOp::Minus => "rat-",
                        BinOp::Times => "rat*",
                        BinOp::Divide => "rat/", // Use rat/ for division (handles both integers and rationals)
                        BinOp::Leq => "rat<=",
                        BinOp::Geq => "rat>=",
                        BinOp::Lt => "rat<",
                        BinOp::Gt => "rat>",
                        BinOp::Equal => "pyret:equal?", // Use polymorphic equality
                        BinOp::Neq => "pyret:not-equal?",
                        BinOp::And => "and",
                        BinOp::Or => "or",
                        BinOp::Spaceship => "pyret:spaceship",
                        BinOp::Roughly => "pyret:roughly-equal?",
                        BinOp::Caret => "expt",
                    }
                } else {
                    match op {
                        BinOp::Plus => "pyret:+", // Use runtime polymorphic + for strings and numbers
                        BinOp::Minus => "-",
                        BinOp::Times => "*",
                        BinOp::Divide => "/",
                        BinOp::Leq => "<=",
                        BinOp::Geq => ">=",
                        BinOp::Lt => "<",
                        BinOp::Gt => ">",
                        BinOp::Equal => "pyret:equal?", // Use polymorphic equality for all types
                        BinOp::Neq => "pyret:not-equal?",
                        BinOp::And => "and",
                        BinOp::Or => "or",
                        BinOp::Spaceship => "pyret:spaceship",
                        BinOp::Roughly => "pyret:roughly-equal?",
                        BinOp::Caret => "expt",
                    }
                };

                let left_str = self.compile_expr(left)?;
                let right_str = self.compile_expr(right)?;

                Ok(format!("({} {} {})", op_str, left_str, right_str))
            }

            Expr::SApp { _fun, args, .. } => {
                // Check if this is a method call (obj.method(...))
                if let Expr::SDot { obj, field, .. } = &**_fun {
                    // This is a method call: obj.method(args)
                    // We need to determine which data type this belongs to and call the dispatch function

                    // Compile the object
                    let obj_code = self.compile_expr(obj)?;

                    // Try to find which data type has this method
                    let mut dispatch_fn_name = None;
                    for (data_name, info) in &self.data_types {
                        if info.methods.contains(field) {
                            dispatch_fn_name = Some(format!(
                                "{}${}",
                                self.mangle_name(data_name),
                                self.mangle_name(field)
                            ));
                            break;
                        }
                    }

                    if let Some(fn_name) = dispatch_fn_name {
                        // Found a dispatch function - use it
                        let args_str = args
                            .iter()
                            .map(|arg| self.compile_expr(arg))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(" ");

                        if args.is_empty() {
                            return Ok(format!("({} {})", fn_name, obj_code));
                        } else {
                            return Ok(format!("({} {} {})", fn_name, obj_code, args_str));
                        }
                    } else {
                        // No data type dispatch found - use runtime method dispatch
                        // This handles built-in types (lists, strings, numbers, etc.)
                        let args_str = args
                            .iter()
                            .map(|arg| self.compile_expr(arg))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(" ");

                        let method_name = field;
                        if args.is_empty() {
                            return Ok(format!(
                                "(pyret:method-call {} \"{}\")",
                                obj_code, method_name
                            ));
                        } else {
                            return Ok(format!(
                                "(pyret:method-call {} \"{}\" {})",
                                obj_code, method_name, args_str
                            ));
                        }
                    }
                }

                // Check if this is a call to a Pyret builtin function or module-scoped function
                if let Expr::SId {
                    id: Name::SName { s, .. },
                    ..
                } = &**_fun
                {
                    // Map Pyret builtin names to runtime function names
                    let runtime_name = match s.as_str() {
                        "print" => Some("pyret:print"),
                        "raise" => Some("pyret:raise"),
                        "num-modulo" => Some("pyret:num-modulo"),
                        "num-remainder" => Some("pyret:num-remainder"),
                        "num-quotient" => Some("pyret:num-quotient"),
                        "num-floor" => Some("pyret:num-floor"),
                        "num-ceiling" => Some("pyret:num-ceiling"),
                        "num-truncate" => Some("pyret:num-truncate"),
                        "num-round" => Some("pyret:num-round"),
                        _ => None,
                    };

                    if let Some(runtime_fn) = runtime_name {
                        // Compile as runtime library call
                        let args_str = args
                            .iter()
                            .map(|arg| self.compile_expr(arg))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(" ");
                        return Ok(format!("({} {})", runtime_fn, args_str));
                    } else if self.toplevel_functions.contains(s) {
                        // This is a call to a user-defined top-level function
                        // Apply module namespacing
                        let func_name = self.namespace_function_name(s);
                        let args_str = args
                            .iter()
                            .map(|arg| self.compile_expr(arg))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(" ");

                        if args.is_empty() {
                            return Ok(format!("({})", func_name));
                        } else {
                            return Ok(format!("({} {})", func_name, args_str));
                        }
                    }
                    // Fall through to regular function application for parameters/local variables
                }

                // Regular function application (for complex function expressions)
                let func_str = self.compile_expr(_fun)?;
                let args_str = args
                    .iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");

                if args.is_empty() {
                    Ok(format!("({})", func_str))
                } else {
                    Ok(format!("({} {})", func_str, args_str))
                }
            }

            Expr::SIfElse {
                branches, _else, ..
            } => {
                // Handle if-else with multiple branches (else-if chains)
                if branches.is_empty() {
                    return Err("If expression has no branches".to_string());
                }

                // Build nested if-else chain from branches
                fn build_if_chain(
                    compiler: &mut SchemeCompiler,
                    branches: &[crate::ast::IfBranch],
                    else_expr: &Box<Expr>,
                    index: usize,
                ) -> Result<String, String> {
                    if index >= branches.len() {
                        // No more branches, compile the else
                        return compiler.compile_expr(else_expr);
                    }

                    let branch = &branches[index];
                    let test_str = compiler.compile_expr(&branch.test)?;
                    let body_str = compiler.compile_expr(&branch.body)?;
                    let else_str = build_if_chain(compiler, branches, else_expr, index + 1)?;

                    Ok(format!("(if {} {} {})", test_str, body_str, else_str))
                }

                build_if_chain(self, branches, _else, 0)
            }

            Expr::SIf { branches, .. } => {
                // If without explicit else - build chain with #f as final else
                if branches.is_empty() {
                    return Err("If expression has no branches".to_string());
                }

                // Build nested if-else chain from branches, with #f as the final else
                fn build_if_chain_no_else(
                    compiler: &mut SchemeCompiler,
                    branches: &[crate::ast::IfBranch],
                    index: usize,
                ) -> Result<String, String> {
                    if index >= branches.len() {
                        // No more branches, use #f as default else
                        return Ok("#f".to_string());
                    }

                    let branch = &branches[index];
                    let test_str = compiler.compile_expr(&branch.test)?;
                    let body_str = compiler.compile_expr(&branch.body)?;
                    let else_str = build_if_chain_no_else(compiler, branches, index + 1)?;

                    Ok(format!("(if {} {} {})", test_str, body_str, else_str))
                }

                build_if_chain_no_else(self, branches, 0)
            }

            Expr::SWhen { test, block, .. } => {
                // When expression: executes block if test is true, otherwise returns nothing
                // Compiles to: (if test block nothing)
                let test_str = self.compile_expr(test)?;
                let block_str = self.compile_expr(block)?;
                Ok(format!("(if {} {} nothing)", test_str, block_str))
            }

            Expr::SIfPipe { branches, .. } => {
                // If-pipe without else: ask | test: body | test2: body2 end
                // Compiles to nested if-else chain with #f as final else
                if branches.is_empty() {
                    return Err("If-pipe expression has no branches".to_string());
                }

                fn build_pipe_chain(
                    compiler: &mut SchemeCompiler,
                    branches: &[crate::ast::IfPipeBranch],
                    index: usize,
                ) -> Result<String, String> {
                    if index >= branches.len() {
                        return Ok("#f".to_string());
                    }

                    let branch = &branches[index];
                    let test_str = compiler.compile_expr(&branch.test)?;
                    let body_str = compiler.compile_expr(&branch.body)?;
                    let else_str = build_pipe_chain(compiler, branches, index + 1)?;

                    Ok(format!("(if {} {} {})", test_str, body_str, else_str))
                }

                build_pipe_chain(self, branches, 0)
            }

            Expr::SIfPipeElse {
                branches, _else, ..
            } => {
                // If-pipe with else: ask | test: body | test2: body2 | otherwise: else_body end
                // Compiles to nested if-else chain
                if branches.is_empty() {
                    return Err("If-pipe-else expression has no branches".to_string());
                }

                fn build_pipe_chain_else(
                    compiler: &mut SchemeCompiler,
                    branches: &[crate::ast::IfPipeBranch],
                    else_expr: &Box<Expr>,
                    index: usize,
                ) -> Result<String, String> {
                    if index >= branches.len() {
                        return compiler.compile_expr(else_expr);
                    }

                    let branch = &branches[index];
                    let test_str = compiler.compile_expr(&branch.test)?;
                    let body_str = compiler.compile_expr(&branch.body)?;
                    let else_str = build_pipe_chain_else(compiler, branches, else_expr, index + 1)?;

                    Ok(format!("(if {} {} {})", test_str, body_str, else_str))
                }

                build_pipe_chain_else(self, branches, _else, 0)
            }

            Expr::SFun {
                name, args, body, ..
            } => {
                // Track this as a top-level function
                self.toplevel_functions.insert(name.clone());

                let params_str = args
                    .iter()
                    .map(|b| self.compile_bind_name(b))
                    .collect::<Vec<_>>()
                    .join(" ");

                self.indent_level += 1;
                let body_str = self.compile_expr(body)?;
                self.indent_level -= 1;

                Ok(format!(
                    "(define ({} {})\n{}  {})",
                    self.namespace_function_name(name),
                    params_str,
                    self.indent(),
                    body_str
                ))
            }

            Expr::SLam { args, body, .. } => {
                let params_str = args
                    .iter()
                    .map(|b| self.compile_bind_name(b))
                    .collect::<Vec<_>>()
                    .join(" ");

                self.indent_level += 1;
                let body_str = self.compile_expr(body)?;
                self.indent_level -= 1;

                Ok(format!(
                    "(lambda ({})\n{}  {})",
                    params_str,
                    self.indent(),
                    body_str
                ))
            }

            Expr::SMethod { args, body, .. } => {
                // Methods are lambdas with an explicit self parameter
                let params_str = args
                    .iter()
                    .map(|b| self.compile_bind_name(b))
                    .collect::<Vec<_>>()
                    .join(" ");

                self.indent_level += 1;
                let body_str = self.compile_expr(body)?;
                self.indent_level -= 1;

                Ok(format!(
                    "(lambda ({})\n{}  {})",
                    params_str,
                    self.indent(),
                    body_str
                ))
            }

            Expr::SUserBlock { body, .. } => {
                // User blocks are just syntactic - compile the inner block
                self.compile_expr(body)
            }

            Expr::SBlock { stmts, .. } => {
                // Compile block as a sequence of statements
                if stmts.is_empty() {
                    return Ok("#<void>".to_string());
                }

                if stmts.len() == 1 {
                    return self.compile_expr(&stmts[0]);
                }

                // Check if we have any SLet statements - if so, use let* for proper scoping
                let has_let = stmts
                    .iter()
                    .any(|stmt| matches!(&**stmt, Expr::SLet { .. }));

                if has_let {
                    // Build nested let* expressions to properly handle interleaved bindings and statements
                    // Each SLet creates a new scope, and following statements execute in that scope
                    fn build_nested_lets(
                        stmts: &[Box<Expr>],
                        compiler: &mut SchemeCompiler,
                    ) -> Result<String, String> {
                        if stmts.is_empty() {
                            return Ok("#<void>".to_string());
                        }

                        // Find the first SLet
                        if let Some((idx, _)) = stmts
                            .iter()
                            .enumerate()
                            .find(|(_, s)| matches!(&***s, Expr::SLet { .. }))
                        {
                            // Compile any statements before this let
                            let before: Result<Vec<_>, _> = stmts[..idx]
                                .iter()
                                .map(|s| compiler.compile_expr(s))
                                .collect();
                            let before = before?;

                            let (var_name, value_str) =
                                if let Expr::SLet { name, value, .. } = &*stmts[idx] {
                                    let vn = compiler.compile_bind_name(name);
                                    let vs = compiler.compile_expr(value)?;
                                    (vn, vs)
                                } else {
                                    unreachable!()
                                };

                            // Recursively build the rest
                            let rest = build_nested_lets(&stmts[idx + 1..], compiler)?;

                            let binding_part =
                                format!("(let* (({}  {})) {})", var_name, value_str, rest);

                            // Wrap with begin if there are statements before
                            if before.is_empty() {
                                Ok(binding_part)
                            } else {
                                Ok(format!("(begin {} {})", before.join(" "), binding_part))
                            }
                        } else {
                            // No more SLets, compile remaining statements
                            if stmts.len() == 1 {
                                compiler.compile_expr(&stmts[0])
                            } else {
                                let exprs: Result<Vec<_>, _> =
                                    stmts.iter().map(|s| compiler.compile_expr(s)).collect();
                                Ok(format!("(begin {})", exprs?.join(" ")))
                            }
                        }
                    }

                    build_nested_lets(stmts, self)
                } else {
                    // No let statements, use regular begin
                    let mut output = String::from("(begin\n");
                    self.indent_level += 1;

                    for (i, stmt) in stmts.iter().enumerate() {
                        output.push_str(&format!("{}{}", self.indent(), self.compile_expr(stmt)?));
                        if i < stmts.len() - 1 {
                            output.push('\n');
                        }
                    }

                    self.indent_level -= 1;
                    output.push(')');
                    Ok(output)
                }
            }

            Expr::SParen { expr, .. } => {
                // Parentheses are just for grouping in Pyret, compile the inner expression
                self.compile_expr(expr)
            }

            Expr::SLet { name, value, .. } => {
                // Variable binding: x = 5
                // In Scheme, this becomes (define x 5)
                let var_name = self.compile_bind_name(name);
                let value_str = self.compile_expr(value)?;
                Ok(format!("(define {} {})", var_name, value_str))
            }

            Expr::SVar { name, value, .. } => {
                // Mutable variable binding: var x = 5
                // In Scheme, this becomes (define x (make-box 5))
                let var_name = self.compile_bind_name(name);
                // Track this as a mutable variable
                self.mutable_vars.insert(var_name.clone());
                let value_str = self.compile_expr(value)?;
                Ok(format!("(define {} (make-box {}))", var_name, value_str))
            }

            Expr::SAssign { id, value, .. } => {
                // Assignment: x := 10
                // In Scheme, this becomes (box-set! x 10)
                let var_name = self.compile_name(id);
                let value_str = self.compile_expr(value)?;
                Ok(format!("(box-set! {} {})", var_name, value_str))
            }

            Expr::STuple { fields, .. } => {
                // Tuple literal: {1; 2; 3}
                // In Scheme: (vector 1 2 3) or use runtime helper (pyret:tuple 1 2 3)
                let fields_str = fields
                    .iter()
                    .map(|f| self.compile_expr(f))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                Ok(format!("(vector {})", fields_str))
            }

            Expr::STupleGet { tup, index, .. } => {
                // Tuple access: t.{0}
                // In Scheme: (vector-ref t 0)
                let tup_str = self.compile_expr(tup)?;
                Ok(format!("(vector-ref {} {})", tup_str, index))
            }

            Expr::SObj { fields, .. } => {
                // Object literal: {x: 1, y: 2}
                // In Scheme: (pyret:make-object-literal "x" 1 "y" 2)
                let mut field_value_pairs = Vec::new();

                for member in fields {
                    match member {
                        crate::ast::Member::SDataField { name, value, .. } => {
                            // Field name as string
                            field_value_pairs.push(format!("\"{}\"", name));
                            // Field value (compiled expression)
                            field_value_pairs.push(self.compile_expr(value)?);
                        }
                        crate::ast::Member::SMutableField { name, value, .. } => {
                            // For now, treat mutable fields the same as data fields
                            // TODO: proper mutable field support
                            field_value_pairs.push(format!("\"{}\"", name));
                            field_value_pairs.push(self.compile_expr(value)?);
                        }
                        crate::ast::Member::SMethodField { .. } => {
                            return Err("Object methods are not yet supported".to_string());
                        }
                    }
                }

                if field_value_pairs.is_empty() {
                    Ok("(pyret:make-object-literal)".to_string())
                } else {
                    Ok(format!(
                        "(pyret:make-object-literal {})",
                        field_value_pairs.join(" ")
                    ))
                }
            }

            Expr::SConstruct {
                constructor,
                values,
                ..
            } => {
                // Construct expression: [list: 1, 2, 3] or [list-set: a, b]
                // Get the original constructor name (before mangling)
                let constructor_name = if let Expr::SId {
                    id: Name::SName { s, .. },
                    ..
                } = &**constructor
                {
                    s.clone()
                } else {
                    return Err(
                        "Construct expression requires simple identifier constructor".to_string(),
                    );
                };

                // Map common constructors to runtime functions
                let runtime_fn = match constructor_name.as_str() {
                    "list" => "pyret:construct-list",
                    "list-set" => "pyret:construct-list-set",
                    "tree-set" => "pyret:construct-tree-set", // TODO: implement
                    // Could add more: array, etc.
                    _ => return Err(format!("Unknown constructor: {}", constructor_name)),
                };

                // Compile the values
                let values_str = values
                    .iter()
                    .map(|v| self.compile_expr(v))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");

                if values.is_empty() {
                    Ok(format!("({})", runtime_fn))
                } else {
                    Ok(format!("({} {})", runtime_fn, values_str))
                }
            }

            // ===== Testing =====
            Expr::SCheck { l, name, body, .. } => {
                if !self.enable_checks {
                    // When checks are disabled, return nothing/void
                    return Ok("(begin)".to_string());
                }

                // Extract location information
                let file = if let Some(ref registry) = self.file_registry {
                    registry
                        .get_filename(l.file_id)
                        .map(|s| s.as_str())
                        .unwrap_or("unknown")
                } else {
                    "unknown"
                };
                let start_line = l.start_line;
                let start_col = l.start_column;
                let end_line = l.end_line;
                let end_col = l.end_column;

                // Start check block tracking
                let name_str = if let Some(ref n) = name {
                    format!("\"{}\"", n)
                } else {
                    "#f".to_string()
                };

                let mut result = format!(
                    "(pyret:check-block-start {} \"{}\" {} {} {} {})\n",
                    name_str, file, start_line, start_col, end_line, end_col
                );

                // Compile the check block body (which should contain SCheckTest expressions)
                let body_code = self.compile_expr(body)?;
                result.push_str(&body_code);

                Ok(result)
            }

            Expr::SCheckTest {
                l,
                op,
                refinement,
                left,
                right,
                ..
            } => {
                if !self.enable_checks {
                    // When checks are disabled, return nothing/void
                    return Ok("(begin)".to_string());
                }

                // Extract location information
                let line = l.start_line;
                let col = l.start_column;

                // Compile left expression
                let left_code = self.compile_expr(left)?;

                // Different check operators have different behavior
                match op {
                    CheckOp::SOpIs { .. } => {
                        // is operator - check equality
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;

                            // Use refinement if provided
                            if let Some(ref_expr) = refinement {
                                let ref_code = self.compile_expr(ref_expr)?;
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result ({} left-val right-val) {} {} left-val right-val \"Values not equal\"))",
                                    left_code, right_code, ref_code, line, col
                                ))
                            } else {
                                // Default equality check
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (pyret:equal? left-val right-val) {} {} left-val right-val \"Values not equal\"))",
                                    left_code, right_code, line, col
                                ))
                            }
                        } else {
                            Err("is operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpIsNot { .. } => {
                        // is-not operator - check inequality
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;

                            // Use refinement if provided
                            if let Some(ref_expr) = refinement {
                                let ref_code = self.compile_expr(ref_expr)?;
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (not ({} left-val right-val)) {} {} left-val right-val \"Values are equal\"))",
                                    left_code, right_code, ref_code, line, col
                                ))
                            } else {
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (not (pyret:equal? left-val right-val)) {} {} left-val right-val \"Values are equal\"))",
                                    left_code, right_code, line, col
                                ))
                            }
                        } else {
                            Err("is-not operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpIsRoughly { .. } => {
                        // is-roughly (is=~) operator - rough equality for numbers
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (pyret:is-roughly left-val right-val) {} {} left-val right-val \"Values not roughly equal\"))",
                                left_code, right_code, line, col
                            ))
                        } else {
                            Err("is-roughly operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpIsNotRoughly { .. } => {
                        // is-not-roughly operator - rough inequality
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (not (pyret:is-roughly left-val right-val)) {} {} left-val right-val \"Values are roughly equal\"))",
                                left_code, right_code, line, col
                            ))
                        } else {
                            Err("is-not-roughly operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpIsOp { op: op_name, .. } => {
                        // is-op operators (is==, is<=>)
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;

                            // Use refinement if provided
                            if let Some(ref_expr) = refinement {
                                let ref_code = self.compile_expr(ref_expr)?;
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result ({} left-val right-val) {} {} left-val right-val \"Values not equal ({})\"))",
                                    left_code, right_code, ref_code, line, col, op_name
                                ))
                            } else {
                                // For custom operators, just use equal? for now
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (pyret:equal? left-val right-val) {} {} left-val right-val \"Values not equal ({})\"))",
                                    left_code, right_code, line, col, op_name
                                ))
                            }
                        } else {
                            Err("is-op operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpIsNotOp { op: op_name, .. } => {
                        // is-not-op operators (is-not==, is-not<=>)
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;

                            // Use refinement if provided
                            if let Some(ref_expr) = refinement {
                                let ref_code = self.compile_expr(ref_expr)?;
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (not ({} left-val right-val)) {} {} left-val right-val \"Values are equal ({})\"))",
                                    left_code, right_code, ref_code, line, col, op_name
                                ))
                            } else {
                                Ok(format!(
                                    "(let ((left-val {}) (right-val {}))\n  (pyret:check-test-result (not (pyret:equal? left-val right-val)) {} {} left-val right-val \"Values are equal ({})\"))",
                                    left_code, right_code, line, col, op_name
                                ))
                            }
                        } else {
                            Err("is-not-op operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpSatisfies { .. } => {
                        // satisfies operator - check if value satisfies predicate
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let ((left-val {}) (predicate {}))\n  (pyret:check-test-result (pyret:satisfies left-val predicate) {} {} left-val predicate \"Predicate not satisfied\"))",
                                left_code, right_code, line, col
                            ))
                        } else {
                            Err("satisfies operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpSatisfiesNot { .. } => {
                        // satisfies-not (violates) operator
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let ((left-val {}) (predicate {}))\n  (pyret:check-test-result (not (pyret:satisfies left-val predicate)) {} {} left-val predicate \"Predicate satisfied\"))",
                                left_code, right_code, line, col
                            ))
                        } else {
                            Err("satisfies-not operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpRaises { .. } => {
                        // raises operator - check if expression raises expected exception
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let* ((expected-msg {})\n       (result (pyret:catch-exception (lambda () {})))\n       (is-error (eq? (car result) 'error))\n       (actual-msg (if is-error (cadr result) \"\"))\n       (messages-match (and is-error (string=? actual-msg expected-msg))))\n  (pyret:check-test-result messages-match {} {} actual-msg expected-msg \"Expected exception not raised\"))",
                                right_code, left_code, line, col
                            ))
                        } else {
                            Err("raises operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpRaisesNot { .. } => {
                        // raises-not (does-not-raise) operator
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let* ((result (pyret:catch-exception (lambda () {})))\n       (is-ok (eq? (car result) 'ok)))\n  (pyret:check-test-result is-ok {} {} (if is-ok \"no error\" (cadr result)) {} \"Unexpected exception raised\"))",
                                left_code, line, col, right_code
                            ))
                        } else {
                            Err("raises-not operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpRaisesOther { .. } => {
                        // raises-other operator - check if exception is different from expected
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let* ((expected-msg {})\n       (result (pyret:catch-exception (lambda () {})))\n       (is-error (eq? (car result) 'error))\n       (actual-msg (if is-error (cadr result) \"\"))\n       (messages-differ (and is-error (not (string=? actual-msg expected-msg)))))\n  (pyret:check-test-result messages-differ {} {} actual-msg expected-msg \"Exception message matched when it shouldn't\"))",
                                right_code, left_code, line, col
                            ))
                        } else {
                            Err("raises-other operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpRaisesSatisfies { .. } => {
                        // raises-satisfies operator - exception message satisfies predicate
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let* ((predicate {})\n       (result (pyret:catch-exception (lambda () {})))\n       (is-error (eq? (car result) 'error))\n       (actual-msg (if is-error (cadr result) \"\"))\n       (satisfies (and is-error (pyret:satisfies actual-msg predicate))))\n  (pyret:check-test-result satisfies {} {} actual-msg predicate \"Exception message doesn't satisfy predicate\"))",
                                right_code, left_code, line, col
                            ))
                        } else {
                            Err("raises-satisfies operator requires right operand".to_string())
                        }
                    }

                    CheckOp::SOpRaisesViolates { .. } => {
                        // raises-violates operator - exception message violates predicate
                        if let Some(right_expr) = right {
                            let right_code = self.compile_expr(right_expr)?;
                            Ok(format!(
                                "(let* ((predicate {})\n       (result (pyret:catch-exception (lambda () {})))\n       (is-error (eq? (car result) 'error))\n       (actual-msg (if is-error (cadr result) \"\"))\n       (violates (and is-error (not (pyret:satisfies actual-msg predicate)))))\n  (pyret:check-test-result violates {} {} actual-msg predicate \"Exception message satisfies predicate when it shouldn't\"))",
                                right_code, left_code, line, col
                            ))
                        } else {
                            Err("raises-violates operator requires right operand".to_string())
                        }
                    }
                }
            }

            // ===== Data Declarations =====
            Expr::SData {
                name: data_name,
                variants,
                check,
                ..
            } => {
                // Check if this data type shadows a builtin
                if self.builtin_types.contains(data_name) {
                    return Err(format!(
                        "The declaration of `{}` shadows the declaration of a built-in of the same name, defined at <builtin builtin://{}>",
                        data_name,
                        data_name.to_lowercase()
                    ));
                }

                let mut result = String::new();
                let mut variant_names = Vec::new();
                let mut method_names = HashSet::new();

                // FIRST PASS: Collect all variant names and method names
                for variant in variants {
                    let (variant_name, with_members) = match variant {
                        crate::ast::Variant::SSingletonVariant {
                            name, with_members, ..
                        } => (name, with_members),
                        crate::ast::Variant::SVariant {
                            name, with_members, ..
                        } => (name, with_members),
                    };

                    variant_names.push(variant_name.clone());
                    self.variant_to_datatype
                        .insert(variant_name.clone(), data_name.clone());

                    // Collect method names
                    for member in with_members {
                        if let crate::ast::Member::SMethodField { name, .. } = member {
                            method_names.insert(name.clone());
                        }
                    }
                }

                // Store data type information BEFORE generating methods
                // This allows method bodies to reference other methods via dispatch
                self.data_types.insert(
                    data_name.clone(),
                    DataTypeInfo {
                        variants: variant_names.clone(),
                        methods: method_names.clone(),
                    },
                );

                // SECOND PASS: Generate constructor functions and methods for each variant
                for variant in variants {
                    match variant {
                        crate::ast::Variant::SSingletonVariant {
                            name: variant_name,
                            with_members,
                            ..
                        } => {
                            // Singleton variant: Generate both a value and a function
                            // Value: (define empty-value '(empty))
                            // Function: (define (empty) empty-value)
                            let constructor_name = self.namespace_function_name(variant_name);
                            let value_name = format!("{}-value", constructor_name);

                            // Generate the value definition
                            result.push_str(&format!(
                                "(define {} '({}))\n",
                                value_name, variant_name
                            ));

                            // Generate the function that returns the value
                            result.push_str(&format!(
                                "(define ({}) {})\n",
                                constructor_name, value_name
                            ));

                            // Track this as a singleton constructor (used as value)
                            self.singleton_constructors.insert(variant_name.clone());

                            // Also track as a toplevel function (can be called)
                            self.toplevel_functions.insert(variant_name.clone());

                            // Generate predicate: (define (is-red? x) (and (pair? x) (eq? (car x) 'red)))
                            // Use full mangling and namespacing for the predicate name to match how it's called
                            let predicate_name =
                                self.namespace_function_name(&format!("is-{}", variant_name));
                            result.push_str(&format!(
                                "(define ({} x) (and (pair? x) (eq? (car x) '{})))\n",
                                predicate_name, variant_name
                            ));

                            // Track the predicate as a toplevel function
                            self.toplevel_functions
                                .insert(format!("is-{}", variant_name));

                            // Track that this variant has no fields
                            self.variant_fields
                                .insert(variant_name.clone(), VariantInfo { fields: vec![] });

                            // Generate methods for this variant
                            self.compile_variant_methods(&mut result, variant_name, with_members)?;
                        }
                        crate::ast::Variant::SVariant {
                            name: variant_name,
                            members,
                            with_members,
                            ..
                        } => {
                            // Variant with fields: (define (point x y) (list 'point x y))
                            let constructor_name = self.namespace_function_name(variant_name);

                            // Extract field names from members
                            let field_names: Vec<String> = members
                                .iter()
                                .map(|member| {
                                    // Get the original field name (before mangling) from the bind
                                    match &member.bind {
                                        crate::ast::Bind::SBind { id, .. } => match id {
                                            Name::SName { s, .. } => s.clone(),
                                            _ => self.compile_name(id),
                                        },
                                        _ => self.compile_bind_name(&member.bind),
                                    }
                                })
                                .collect();

                            // Track field information for this variant
                            self.variant_fields.insert(
                                variant_name.clone(),
                                VariantInfo {
                                    fields: field_names.clone(),
                                },
                            );

                            // Track this constructor as a toplevel function
                            self.toplevel_functions.insert(variant_name.clone());

                            // Generate mangled parameter names
                            let mangled_params: Vec<String> = members
                                .iter()
                                .map(|member| self.compile_bind_name(&member.bind))
                                .collect();

                            let params = mangled_params.join(" ");
                            let fields = mangled_params.join(" ");

                            result.push_str(&format!(
                                "(define ({} {}) (list '{} {}))\n",
                                constructor_name, params, variant_name, fields
                            ));

                            // Generate predicate: (define (is-link? x) (and (pair? x) (eq? (car x) 'link)))
                            // Use full mangling and namespacing for the predicate name to match how it's called
                            let predicate_name =
                                self.namespace_function_name(&format!("is-{}", variant_name));
                            result.push_str(&format!(
                                "(define ({} x) (and (pair? x) (eq? (car x) '{})))\n",
                                predicate_name, variant_name
                            ));

                            // Track the predicate as a toplevel function
                            self.toplevel_functions
                                .insert(format!("is-{}", variant_name));

                            // Generate methods for this variant
                            self.compile_variant_methods(&mut result, variant_name, with_members)?;
                        }
                    }
                }

                // Generate dispatch functions for each method
                // For each method, create a function that dispatches based on the variant tag
                for method_name in &method_names {
                    let dispatch_fn_name = format!(
                        "{}${}",
                        self.mangle_name(data_name),
                        self.mangle_name(method_name)
                    );

                    result.push_str(&format!("(define ({} obj . args)\n", dispatch_fn_name));
                    result.push_str("  (let ((tag (car obj)))\n");
                    result.push_str("    (cond\n");

                    // Generate case for each variant
                    for variant_name in &variant_names {
                        let variant_method_fn = format!(
                            "{}${}",
                            self.mangle_name(variant_name),
                            self.mangle_name(method_name)
                        );
                        result.push_str(&format!(
                            "      ((eq? tag '{}) (apply {} obj args))\n",
                            variant_name, variant_method_fn
                        ));
                    }

                    result.push_str("      (else (error \"Unknown variant for method\" tag)))))\n");
                    result.push_str("\n");
                }

                // TODO: Generate shared methods (from shared_members)
                // For now, we'll skip shared members

                // Compile where block (check) if present
                if let Some(check_expr) = check {
                    if self.enable_checks {
                        result.push_str(&self.compile_expr(check_expr)?);
                        result.push('\n');
                    }
                }

                // Data declarations don't produce a value themselves
                // Return (begin) to represent "no value"
                result.push_str("(begin)");
                Ok(result)
            }

            Expr::SDataExpr { .. } => {
                // Data expression (data as value) - not yet implemented
                Err("Data expressions are not yet supported".to_string())
            }

            Expr::SDot { obj, field, .. } => {
                // Field access: obj.field
                // This can be one of:
                // 1. Qualified module name: M.foo where M is an import alias
                // 2. Data variant field access (using field index)
                // 3. Object field access (using field name)

                // Check if obj is a simple identifier that matches an import alias
                if let Expr::SId {
                    id: Name::SName { s, .. },
                    ..
                } = &**obj
                {
                    if let Some(module_uri) = self.imports.get(s) {
                        // This is a qualified name: M.foo
                        // Generate the fully-qualified function name
                        let prefix = self.get_module_prefix(module_uri);
                        return Ok(format!("{}__{}", prefix, self.mangle_name(field)));
                    }
                }

                let obj_code = self.compile_expr(obj)?;

                // Try to find which variant(s) have this field
                let mut field_indices = Vec::new();
                for (variant_name, info) in &self.variant_fields {
                    if let Some(index) = info.fields.iter().position(|f| f == field) {
                        field_indices.push((variant_name.clone(), index));
                    }
                }

                if !field_indices.is_empty() {
                    // Found in data variants - use data-field access
                    // For now, assume all variants with this field name have it at the same position
                    // (This is a simplification - Pyret allows different variants to have different fields)
                    let field_index = field_indices[0].1;
                    Ok(format!("(pyret:data-field {} {})", obj_code, field_index))
                } else {
                    // Not found in data variants - assume it's an object field
                    Ok(format!("(pyret:object-get {} \"{}\")", obj_code, field))
                }
            }

            // ===== Pattern Matching =====
            Expr::SCases { val, branches, .. } => {
                // Cases expression without else - add default error
                self.compile_cases(val, branches, None)
            }

            Expr::SCasesElse {
                val,
                branches,
                _else,
                ..
            } => {
                // Cases expression with else clause
                self.compile_cases(val, branches, Some(_else))
            }

            // ===== For Loops =====
            Expr::SFor {
                iterator,
                bindings,
                body,
                ..
            } => self.compile_for(iterator, bindings, body),

            _ => Err(format!("Unsupported expression type: {:?}", expr)),
        }
    }

    /// Compile methods for a data variant
    /// Each method becomes a function: (define (variant-name-method-name self arg1 arg2 ...) body)
    fn compile_variant_methods(
        &mut self,
        result: &mut String,
        variant_name: &str,
        with_members: &[crate::ast::Member],
    ) -> Result<(), String> {
        use crate::ast::Member;

        for member in with_members {
            match member {
                Member::SMethodField {
                    name,
                    params: _,
                    args,
                    body,
                    ..
                } => {
                    // Generate method function name: variant-name$method-name
                    let method_fn_name = format!(
                        "{}${}",
                        self.mangle_name(variant_name),
                        self.mangle_name(name)
                    );

                    // Compile parameters - first param is 'self'
                    let param_names: Vec<String> =
                        args.iter().map(|arg| self.compile_bind_name(arg)).collect();

                    // Compile method body
                    self.indent_level += 1;
                    let body_code = self.compile_expr(body)?;
                    self.indent_level -= 1;

                    // Generate function definition
                    // (define (variant$method self param1 param2 ...) body)
                    let all_params = param_names.join(" ");

                    result.push_str(&format!(
                        "(define ({} {}) {})\n",
                        method_fn_name, all_params, body_code
                    ));

                    // Track this method for later dispatch
                    // Store mapping: (variant_name, method_name) -> function_name
                    // This will be used when compiling method calls
                }
                Member::SDataField { .. } => {
                    // Regular data fields are not methods, skip
                }
                Member::SMutableField { .. } => {
                    // Mutable fields, skip for now
                }
            }
        }

        Ok(())
    }

    /// Compile a cases expression (pattern matching)
    fn compile_cases(
        &mut self,
        val: &Box<Expr>,
        branches: &[crate::ast::CasesBranch],
        else_clause: Option<&Box<Expr>>,
    ) -> Result<String, String> {
        // Step 1: Compile the value expression
        let val_code = self.compile_expr(val)?;

        // Step 2: Generate a unique temporary variable to hold the value
        // Use the location to make it unique (avoids conflicts in nested cases)
        let val_var = format!("pyret$cases$val");

        // Step 3: Build the cond expression
        let mut result = format!("(let (({} {}))\n", val_var, val_code);
        result.push_str("  (cond\n");

        // Step 4: Generate each branch
        for branch in branches {
            result.push_str(&self.compile_cases_branch(&val_var, branch)?);
        }

        // Step 5: Add else clause or error
        if let Some(else_expr) = else_clause {
            let else_code = self.compile_expr(else_expr)?;
            result.push_str(&format!("    (else {})\n", else_code));
        } else {
            result.push_str("    (else (pyret:error \"No case matched\"))\n");
        }

        result.push_str("  ))");
        Ok(result)
    }

    /// Compile a single cases branch
    fn compile_cases_branch(
        &mut self,
        val_var: &str,
        branch: &crate::ast::CasesBranch,
    ) -> Result<String, String> {
        use crate::ast::CasesBranch;

        match branch {
            CasesBranch::SCasesBranch {
                name, args, body, ..
            } => {
                // Branch with field bindings
                // Generate the tag check: (eq? (car val) 'variant-name)
                let mut result = format!("    ((eq? (car {}) '{}) ", val_var, name);

                // If there are pattern bindings, extract fields
                if !args.is_empty() {
                    result.push_str("(let (");

                    for (i, cases_bind) in args.iter().enumerate() {
                        let field_name = self.compile_bind_name(&cases_bind.bind);
                        // Extract field at index i+1 (skip the tag at index 0)
                        result.push_str(&format!(
                            "({} (list-ref {} {}))",
                            field_name,
                            val_var,
                            i + 1
                        ));
                        if i < args.len() - 1 {
                            result.push(' ');
                        }
                    }

                    result.push_str(") ");

                    // Compile the branch body
                    let body_code = self.compile_expr(body)?;
                    result.push_str(&body_code);
                    result.push(')');
                } else {
                    // No bindings, just compile the body
                    let body_code = self.compile_expr(body)?;
                    result.push_str(&body_code);
                }

                result.push_str(")\n");
                Ok(result)
            }

            CasesBranch::SSingletonCasesBranch { name, body, .. } => {
                // Singleton branch (no fields)
                // Just check the tag and compile the body
                let body_code = self.compile_expr(body)?;
                Ok(format!(
                    "    ((eq? (car {}) '{})\n     {})\n",
                    val_var, name, body_code
                ))
            }
        }
    }

    /// Compile a for loop expression
    /// For loops in Pyret are desugared into calls to iterator functions:
    /// - for map(x from lst): body end => map(lam(x): body end, lst)
    /// - for filter(x from lst): body end => filter(lam(x): body end, lst)
    /// - for fold(acc from init, x from lst): body end => fold(lam(acc, x): body end, init, lst)
    /// - for each(x from lst): body end => each(lam(x): body end, lst)
    fn compile_for(
        &mut self,
        iterator: &Box<Expr>,
        bindings: &[crate::ast::ForBind],
        body: &Box<Expr>,
    ) -> Result<String, String> {
        // Get the iterator function name
        // Iterator can be:
        // - Simple identifier: "map", "filter", etc.
        // - Dot access: "lists.map", etc.
        // - Type-instantiated: "map<T>", etc.
        let iterator_code = self.compile_expr(iterator)?;

        // Build the lambda function for the loop body
        // The lambda parameters are the bindings
        let mut lambda_params = Vec::new();
        let mut value_exprs = Vec::new();

        for bind in bindings {
            // Get the binding name
            let param_name = self.compile_bind_name(&bind.bind);
            lambda_params.push(param_name);

            // Compile the 'from' expression (the list/collection being iterated)
            let value_code = self.compile_expr(&bind.value)?;
            value_exprs.push(value_code);
        }

        // Compile the loop body
        let body_code = self.compile_expr(body)?;

        // Build the lambda
        let params_str = lambda_params.join(" ");
        let lambda = format!("(lambda ({}) {})", params_str, body_code);

        // Build the iterator call
        // For single binding: (iterator lambda list)
        // For fold with 2 bindings: (iterator lambda init list)
        // For multiple bindings (cartesian product): (iterator lambda list1 list2 ...)
        let mut result = format!("({} {}", iterator_code, lambda);
        for value_expr in value_exprs {
            result.push(' ');
            result.push_str(&value_expr);
        }
        result.push(')');

        Ok(result)
    }

    fn compile_name(&self, name: &Name) -> String {
        match name {
            Name::SName { s, .. } => self.mangle_name(s),
            Name::SGlobal { s } => format!("global${}", self.mangle_name(s)),
            Name::SUnderscore { .. } => "_".to_string(),
            Name::SModuleGlobal { s } => format!("module${}", self.mangle_name(s)),
            Name::STypeGlobal { s } => format!("type${}", self.mangle_name(s)),
            Name::SAtom { base, serial } => format!("{}${}", self.mangle_name(base), serial),
        }
    }

    fn compile_bind_name(&self, bind: &crate::ast::Bind) -> String {
        match bind {
            crate::ast::Bind::SBind { id, .. } => self.compile_name(id),
            crate::ast::Bind::STupleBind { fields, .. } => {
                // For tuple binds, just use first field for simplicity
                // In a real compiler, you'd destructure
                if let Some(first) = fields.first() {
                    self.compile_bind_name(first)
                } else {
                    "_".to_string()
                }
            }
        }
    }
}

impl Default for SchemeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn make_loc() -> Loc {
        Loc::new(FileId(0), 0, 0, 0, 0, 0, 0)
    }

    #[test]
    fn test_compile_number() {
        let mut compiler = SchemeCompiler::new();
        let expr = Expr::SNum {
            l: make_loc(),
            value: "42".to_string(),
        };
        assert_eq!(compiler.compile_expr(&expr).unwrap(), "42");
    }

    #[test]
    fn test_compile_id() {
        let mut compiler = SchemeCompiler::new();
        let expr = Expr::SId {
            l: make_loc(),
            id: Name::SName {
                l: make_loc(),
                s: "x".to_string(),
            },
        };
        assert_eq!(compiler.compile_expr(&expr).unwrap(), "x");
    }

    #[test]
    fn test_compile_addition() {
        let mut compiler = SchemeCompiler::new();
        let expr = Expr::SOp {
            l: make_loc(),
            op_l: make_loc(),
            op: BinOp::Plus,
            left: Box::new(Expr::SNum {
                l: make_loc(),
                value: "1".to_string(),
            }),
            right: Box::new(Expr::SNum {
                l: make_loc(),
                value: "2".to_string(),
            }),
        };
        assert_eq!(compiler.compile_expr(&expr).unwrap(), "(pyret:+ 1 2)");
    }

    #[test]
    fn test_compile_function_call() {
        let mut compiler = SchemeCompiler::new();
        let expr = Expr::SApp {
            l: make_loc(),
            _fun: Box::new(Expr::SId {
                l: make_loc(),
                id: Name::SName {
                    l: make_loc(),
                    s: "f".to_string(),
                },
            }),
            args: vec![Box::new(Expr::SNum {
                l: make_loc(),
                value: "10".to_string(),
            })],
        };
        assert_eq!(compiler.compile_expr(&expr).unwrap(), "(f 10)");
    }
}
