mod lexer;
mod ast;
mod parser;
mod value;
mod interpreter;
mod compiler;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;
use compiler::Compiler;
use ast::Stmt;
use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 2 {
        let file = &args[1];
        translate_file(file);
        return;
    }
    
    run_embedded();
}

fn parse_file_recursive(path: &str, visited: &mut HashSet<String>) -> Result<Vec<Stmt>, String> {
    if visited.contains(path) {
        return Ok(Vec::new());
    }
    visited.insert(path.to_string());
    
    let code = fs::read_to_string(path)
        .map_err(|e| format!("لا يمكن قراءة {}: {}", path, e))?;
    
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("خطأ لغوي في {}: {}", path, e))?;
    
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()
        .map_err(|e| format!("خطأ نحوي في {}: {}", path, e))?;
    
    let mut result = Vec::new();
    let base_dir = Path::new(path).parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    
    for stmt in stmts {
        match stmt {
            Stmt::Import { path: import_path } => {
                let full_path = base_dir.join(&import_path);
                let full_str = full_path.to_string_lossy().to_string();
                let imported = parse_file_recursive(&full_str, visited)?;
                result.extend(imported);
            }
            other => result.push(other),
        }
    }
    
    Ok(result)
}

fn translate_file(file: &str) {
    println!("=== مترجم زمان ===");
    println!("الملف: {}", file);
    
    let mut visited = HashSet::new();
    let stmts = match parse_file_recursive(file, &mut visited) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\n❌❌❌ خطأ: {}\n", e);
            return;
        }
    };
    
    let mut compiler = Compiler::new();
    let c_code = match compiler.compile(&stmts) {
        Ok(c) => c,
        Err(e) => { eprintln!("❌ خطأ ترجمة: {}", e); return; }
    };
    
    let c_file = format!("{}.c", file.trim_end_matches(".zaman"));
    if let Err(e) = fs::write(&c_file, &c_code) {
        eprintln!("❌ لا يمكن كتابة ملف C: {}", e);
        return;
    }
    println!("✅ تم توليد: {}", c_file);
    println!("📝 الآن ابنِ الملف:");
    println!("   clang {} -o output -lm", c_file);
}

fn run_embedded() {
    let code = r#"
        let x = 10
        let y = 20
        print(x + y)
    "#;
    
    println!("================================");
    println!("   لغة زمان - Zaman Language");
    println!("================================");
    
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("خطأ لغوي: {}", e); return; }
    };
    
    let mut parser = Parser::new(tokens);
    let stmts = match parser.parse() {
        Ok(s) => s,
        Err(e) => { eprintln!("خطأ نحوي: {}", e); return; }
    };
    
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(&stmts) {
        eprintln!("خطأ تنفيذ: {}", e);
    }
}
