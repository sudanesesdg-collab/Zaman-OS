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
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // الوضع 1: cargo run -- ملف.zaman  (ترجمة)
    if args.len() >= 2 {
        let file = &args[1];
        translate_file(file);
        return;
    }
    
    // الوضع 2: cargo run  (تشغيل الكود المضمّن)
    run_embedded();
}

fn translate_file(file: &str) {
    println!("=== مترجم زمان ===");
    println!("الملف: {}", file);
    
    let code = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ لا يمكن قراءة الملف: {}", e);
            return;
        }
    };
    
    // 1. التحليل اللغوي
    let mut lexer = Lexer::new(&code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("❌ خطأ لغوي: {}", e); return; }
    };
    
    // 2. التحليل النحوي
    let mut parser = Parser::new(tokens);
    let stmts = match parser.parse() {
        Ok(s) => s,
        Err(e) => { eprintln!("❌ خطأ نحوي: {}", e); return; }
    };
    
    // 3. الترجمة إلى C
    let mut compiler = Compiler::new();
    let c_code = match compiler.compile(&stmts) {
        Ok(c) => c,
        Err(e) => { eprintln!("❌ خطأ ترجمة: {}", e); return; }
    };
    
    // 4. كتابة ملف C
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
