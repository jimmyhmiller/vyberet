//! Multi-file module compilation
//!
//! Handles dependency resolution, topological sorting, and compilation
//! of Pyret programs with imports.

use crate::ast::{Import, ImportType, Name, Program, Provide};
use crate::codegen::SchemeCompiler;
use crate::tokenizer::Tokenizer;
use crate::{FileRegistry, Parser};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Represents a module with its dependencies
#[derive(Debug, Clone)]
pub struct Module {
    /// Unique URI for this module
    pub uri: String,
    /// Absolute path to source file
    pub source_path: PathBuf,
    /// Last modified time
    pub modified_time: SystemTime,
    /// Parsed program AST
    pub program: Program,
    /// Dependencies (import statements)
    pub dependencies: Vec<Dependency>,
    /// Compiled Scheme code (populated during compilation)
    pub compiled_code: Option<String>,
}

/// A dependency from an import statement
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The import type (file, builtin, etc.)
    pub import_type: ImportType,
    /// Resolved URI of the dependency
    pub uri: String,
}

/// Multi-file module compiler
pub struct ModuleCompiler {
    /// All modules discovered so far (URI -> Module)
    modules: HashMap<String, Module>,
    /// File registry for tracking files
    file_registry: FileRegistry,
    /// Project root directory (for project-relative URIs)
    project_root: PathBuf,
}

impl ModuleCompiler {
    pub fn new(project_root: PathBuf) -> Self {
        ModuleCompiler {
            modules: HashMap::new(),
            file_registry: FileRegistry::new(),
            project_root,
        }
    }

    /// Detect project root by looking for .git directory or Cargo.toml
    pub fn detect_project_root(start_path: &Path) -> PathBuf {
        // Canonicalize the path first
        let canonical = match start_path.canonicalize() {
            Ok(p) => p,
            Err(_) => return start_path.to_path_buf(),
        };

        let mut current = canonical.as_path();

        // If start_path is a file, use its parent directory
        if current.is_file() {
            if let Some(parent) = current.parent() {
                current = parent;
            }
        }

        // Walk up the directory tree
        loop {
            // Check for pyret.toml (Pyret project marker)
            if current.join("pyret.toml").exists() {
                return current.to_path_buf();
            }

            // Check for .git directory (Git repository root)
            if current.join(".git").exists() {
                return current.to_path_buf();
            }

            // Move to parent directory
            match current.parent() {
                Some(parent) => current = parent,
                None => {
                    // Reached filesystem root, use the start path's directory
                    return if start_path.is_file() {
                        start_path.parent().unwrap_or(start_path).to_path_buf()
                    } else {
                        start_path.to_path_buf()
                    };
                }
            }
        }
    }

    /// Compile a program and all its dependencies
    pub fn compile_program(&mut self, entry_point: &Path) -> Result<String, String> {
        // 1. Parse entry point and build dependency graph
        let entry_uri = self.load_module(entry_point)?;

        // 2. Topological sort to get compilation order
        let compile_order = self.topological_sort(&entry_uri)?;

        // 3. Compile all modules in dependency order
        for uri in &compile_order {
            self.compile_module(uri)?;
        }

        // 4. Generate standalone output with all modules
        self.generate_standalone(&compile_order)
    }

    /// Load a module from a file path, recursively loading dependencies
    fn load_module(&mut self, path: &Path) -> Result<String, String> {
        // Convert to absolute path
        let absolute_path = path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path {:?}: {}", path, e))?;

        // Generate project-relative URI
        let relative_path = absolute_path
            .strip_prefix(&self.project_root)
            .map_err(|_| format!("Path {:?} is not within project root {:?}", absolute_path, self.project_root))?;

        let relative_str = relative_path.to_string_lossy().replace('\\', "/");
        let uri = format!("file://{}", relative_str);

        // If already loaded, return
        if self.modules.contains_key(&uri) {
            return Ok(uri);
        }

        // Read source
        let source = fs::read_to_string(&absolute_path)
            .map_err(|e| format!("Failed to read {:?}: {}", absolute_path, e))?;

        // Get modified time
        let modified_time = fs::metadata(&absolute_path)
            .and_then(|m| m.modified())
            .map_err(|e| format!("Failed to get mtime for {:?}: {}", absolute_path, e))?;

        // Parse
        let file_id = self.file_registry.register(absolute_path.to_string_lossy().to_string());
        let mut tokenizer = Tokenizer::new(&source, file_id);
        let tokens = tokenizer.tokenize();
        let mut parser = Parser::new(tokens, file_id);
        let program = parser
            .parse_program()
            .map_err(|e| format!("Parse error in {:?}: {:?}", absolute_path, e))?;

        // Extract dependencies
        let dependencies = self.extract_dependencies(&program, &absolute_path)?;

        // Create module
        let module = Module {
            uri: uri.clone(),
            source_path: absolute_path.clone(),
            modified_time,
            program,
            dependencies: dependencies.clone(),
            compiled_code: None,
        };

        // Store module
        self.modules.insert(uri.clone(), module);

        // Recursively load dependencies
        for dep in dependencies {
            if dep.uri.starts_with("builtin://") {
                // Built-in modules - skip for now
                continue;
            }

            if dep.uri.starts_with("file://") {
                let dep_path_str = dep.uri.strip_prefix("file://").unwrap();
                let dep_path = PathBuf::from(dep_path_str);
                self.load_module(&dep_path)?;
            }
        }

        Ok(uri)
    }

    /// Extract dependencies from a program's imports
    fn extract_dependencies(&self, program: &Program, current_file: &Path) -> Result<Vec<Dependency>, String> {
        let mut dependencies = Vec::new();

        for import in &program.imports {
            let import_type = match import {
                Import::SImport { import, .. } => import,
                Import::SInclude { import, .. } => import,
                Import::SIncludeFrom { .. } => continue, // TODO
                Import::SImportFields { import, .. } => import,
                Import::SImportTypes { import, .. } => import,
            };

            let uri = self.resolve_import_to_uri(import_type, current_file)?;
            dependencies.push(Dependency {
                import_type: import_type.clone(),
                uri,
            });
        }

        Ok(dependencies)
    }

    /// Resolve an import to a URI
    fn resolve_import_to_uri(&self, import: &ImportType, current_file: &Path) -> Result<String, String> {
        match import {
            ImportType::SConstImport { module, .. } => {
                // Builtin module
                Ok(format!("builtin://{}", module))
            }
            ImportType::SSpecialImport { kind, args, .. } => {
                match kind.as_str() {
                    "file" if !args.is_empty() => {
                        let import_path = Path::new(&args[0]);
                        let resolved_path = if import_path.is_absolute() {
                            import_path.to_path_buf()
                        } else {
                            // Resolve relative to current file's directory
                            if let Some(parent) = current_file.parent() {
                                parent.join(import_path)
                            } else {
                                import_path.to_path_buf()
                            }
                        };

                        let absolute = resolved_path
                            .canonicalize()
                            .map_err(|e| format!("Failed to resolve import {:?}: {}", resolved_path, e))?;

                        let relative = absolute
                            .strip_prefix(&self.project_root)
                            .map_err(|_| format!("Import path {:?} is not within project root {:?}", absolute, self.project_root))?;

                        let relative_str = relative.to_string_lossy().replace('\\', "/");
                        Ok(format!("file://{}", relative_str))
                    }
                    _ => Err(format!("Unsupported import kind: {}", kind)),
                }
            }
        }
    }

    /// Topological sort with cycle detection
    fn topological_sort(&self, entry_uri: &str) -> Result<Vec<String>, String> {
        let mut sorted = Vec::new();
        let mut temp_mark = HashSet::new();
        let mut perm_mark = HashSet::new();

        fn visit(
            uri: &str,
            modules: &HashMap<String, Module>,
            sorted: &mut Vec<String>,
            temp_mark: &mut HashSet<String>,
            perm_mark: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Result<(), String> {
            if perm_mark.contains(uri) {
                return Ok(());
            }

            if temp_mark.contains(uri) {
                path.push(uri.to_string());
                return Err(format!(
                    "Circular dependency detected: {}",
                    path.join(" => ")
                ));
            }

            // Skip built-in modules for now
            if uri.starts_with("builtin://") {
                return Ok(());
            }

            let module = modules
                .get(uri)
                .ok_or_else(|| format!("Module not found: {}", uri))?;

            temp_mark.insert(uri.to_string());
            path.push(uri.to_string());

            // Visit dependencies
            for dep in &module.dependencies {
                visit(&dep.uri, modules, sorted, temp_mark, perm_mark, path)?;
            }

            path.pop();
            temp_mark.remove(uri);
            perm_mark.insert(uri.to_string());
            sorted.push(uri.to_string());

            Ok(())
        }

        let mut path = Vec::new();
        visit(
            entry_uri,
            &self.modules,
            &mut sorted,
            &mut temp_mark,
            &mut perm_mark,
            &mut path,
        )?;

        Ok(sorted)
    }

    /// Compile a single module
    fn compile_module(&mut self, uri: &str) -> Result<(), String> {
        // Skip if already compiled
        if let Some(module) = self.modules.get(uri) {
            if module.compiled_code.is_some() {
                return Ok(());
            }
        }

        // Skip built-ins
        if uri.starts_with("builtin://") {
            return Ok(());
        }

        // Get the module (need to clone to avoid borrow issues)
        let module = self
            .modules
            .get(uri)
            .ok_or_else(|| format!("Module not found: {}", uri))?
            .clone();

        // Create compiler with project root
        let mut compiler = SchemeCompiler::with_project_root(self.project_root.clone());
        compiler.set_file_registry(self.file_registry.clone());

        // Set module path
        compiler.set_module_from_path(&module.source_path)?;

        // Compile
        let code = compiler.compile_program(&module.program)?;

        // Store compiled code
        if let Some(m) = self.modules.get_mut(uri) {
            m.compiled_code = Some(code);
        }

        Ok(())
    }

    /// Generate standalone output with all modules
    fn generate_standalone(&self, compile_order: &[String]) -> Result<String, String> {
        let mut output = String::new();

        // Header
        output.push_str("; Generated from Pyret by vyberet compiler\n");
        output.push_str("; Multi-file compilation - all dependencies included\n");
        output.push_str("; Target: R4RS Scheme\n\n");

        // Include runtime.scm
        output.push_str("; ===== Runtime Library =====\n");
        output.push_str("; Loading Pyret runtime\n\n");

        // Try to load runtime.scm from the same directory as the binary
        // or from a standard location relative to the project root
        let runtime_path = self.project_root.join("runtime/runtime.scm");
        if runtime_path.exists() {
            let runtime_code = fs::read_to_string(&runtime_path)
                .map_err(|e| format!("Failed to read runtime.scm: {}", e))?;
            output.push_str(&runtime_code);
            output.push_str("\n\n");
        } else {
            return Err(format!("Runtime not found at {:?}", runtime_path));
        }

        // Add all compiled modules in dependency order
        for uri in compile_order {
            if uri.starts_with("builtin://") {
                continue; // Skip built-ins (they're in the runtime)
            }

            let module = self
                .modules
                .get(uri)
                .ok_or_else(|| format!("Module not found: {}", uri))?;

            if let Some(ref code) = module.compiled_code {
                output.push_str(&format!("; ===== Module: {} =====\n", uri));
                output.push_str(&format!("; Source: {}\n\n", module.source_path.display()));
                output.push_str(code);
                output.push_str("\n\n");
            }
        }

        Ok(output)
    }
}
