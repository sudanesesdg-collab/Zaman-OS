use crate::ast::{Expr, Stmt, BinOp};
use std::collections::HashSet;

pub struct Compiler {
    declared_vars: HashSet<String>,
    functions: HashSet<String>,
    indent: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            declared_vars: HashSet::new(),
            functions: HashSet::new(),
            indent: 0,
        }
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        self.collect_functions(stmts);

        let mut out = String::new();
        out.push_str("// كود مُولَّد تلقائيًا من لغة زمان\n");
        out.push_str("#include <stdio.h>\n");
        out.push_str("#include <stdlib.h>\n");
        out.push_str("#include <string.h>\n");
        out.push_str("#include <math.h>\n");
        out.push_str("#include <time.h>\n\n");

        out.push_str(Self::helper_lib());

        // توليد كل الدوال
        for stmt in stmts {
            if let Stmt::Fn { name, params, body } = stmt {
                out.push_str(&self.compile_function(name, params, body)?);
            }
        }

        // main
        out.push_str("int main(void) {\n");
        self.indent = 1;
        for stmt in stmts {
            match stmt {
                Stmt::Fn { .. } => continue,
                _ => out.push_str(&self.compile_stmt(stmt)?),
            }
        }
        out.push_str("    return 0;\n");
        out.push_str("}\n");

        Ok(out)
    }

    fn collect_functions(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if let Stmt::Fn { name, .. } = stmt {
                self.functions.insert(name.clone());
            }
        }
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<String, String> {
        let mut out = String::new();
        out.push_str(&format!("long long zaman_{}(", name));
        for (i, p) in params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(&format!("long long {}", p));
        }
        out.push_str(") {\n");
        
        let old_indent = self.indent;
        self.indent = 1;
        for stmt in body {
            out.push_str(&self.compile_stmt(stmt)?);
        }
        self.indent = old_indent;
        
        out.push_str("    return 0;\n");
        out.push_str("}\n\n");
        Ok(out)
    }

    fn ind(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<String, String> {
        let indent = self.ind();
        match stmt {
            Stmt::Let { name, value } | Stmt::Var { name, value } => {
                let mut out = String::new();
                if !self.declared_vars.contains(name) {
                    out.push_str(&format!("{}long long {} = ", indent, name));
                    self.declared_vars.insert(name.clone());
                } else {
                    out.push_str(&format!("{}{} = ", indent, name));
                }
                out.push_str(&self.compile_expr(value)?);
                out.push_str(";\n");
                Ok(out)
            }
            Stmt::Assign { name, value } => {
                Ok(format!("{}{} = {};\n", indent, name, self.compile_expr(value)?))
            }
            Stmt::IndexAssign { array, index, value } => {
                Ok(format!("{}{}[{}] = {};\n", 
                    indent, array, 
                    self.compile_expr(index)?, 
                    self.compile_expr(value)?))
            }
            Stmt::If { condition, then_body, else_body } => {
                let mut out = String::new();
                out.push_str(&format!("{}if ({}) {{\n", indent, self.compile_expr(condition)?));
                self.indent += 1;
                for s in then_body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                self.indent -= 1;
                if !else_body.is_empty() {
                    out.push_str(&format!("{}}} else {{\n", indent));
                    self.indent += 1;
                    for s in else_body {
                        out.push_str(&self.compile_stmt(s)?);
                    }
                    self.indent -= 1;
                }
                out.push_str(&format!("{}}}\n", indent));
                Ok(out)
            }
            Stmt::Loop { var, from, to, body } => {
                let mut out = String::new();
                self.declared_vars.insert(var.clone());
                out.push_str(&format!(
                    "{}for (long long {} = {}; {} <= {}; {}++) {{\n",
                    indent, var, self.compile_expr(from)?, var, self.compile_expr(to)?, var
                ));
                self.indent += 1;
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                self.indent -= 1;
                out.push_str(&format!("{}}}\n", indent));
                Ok(out)
            }
            Stmt::While { condition, body } => {
                let mut out = String::new();
                out.push_str(&format!("{}while ({}) {{\n", indent, self.compile_expr(condition)?));
                self.indent += 1;
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                self.indent -= 1;
                out.push_str(&format!("{}}}\n", indent));
                Ok(out)
            }
            Stmt::Return(expr) => {
                match expr {
                    Some(e) => Ok(format!("{}return {};\n", indent, self.compile_expr(e)?)),
                    None => Ok(format!("{}return 0;\n", indent)),
                }
            }
            Stmt::Expr(expr) => {
                Ok(format!("{}{};\n", indent, self.compile_expr(expr)?))
            }
            Stmt::Fn { .. } => Ok(String::new()),
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n) => Ok(n.to_string()),
            Expr::Text(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                Ok(format!("\"{}\"", escaped))
            }
            Expr::Bool(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
            Expr::Ident(name) => Ok(name.clone()),
            Expr::Binary { left, op, right } => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Eq => "==",
                    BinOp::NotEq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                Ok(format!("({} {} {})", l, op_str, r))
            }
            Expr::Call { name, args } => {
                if name == "print" || name == "اطبع" {
                    // نبني printf مباشرة
                    let mut fmt = String::new();
                    let mut fmt_args = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { fmt.push(' '); }
                        let compiled = self.compile_expr(arg)?;
                        if let Expr::Text(_) = arg {
                            fmt.push_str("%s");
                        } else {
                            fmt.push_str("%lld");
                        }
                        fmt_args.push(compiled);
                    }
                    fmt.push_str("\\n");
                    let mut call = format!("printf(\"{}\"", fmt);
                    for a in fmt_args {
                        call.push_str(", ");
                        call.push_str(&a);
                    }
                    call.push(')');
                    return Ok(call);
                }

                let mut compiled_args = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?);
                }
                let args_str = compiled_args.join(", ");
                
                match name.as_str() {
                    "الطول" | "length" => Ok(format!("zaman_len({})", args_str)),
                    "الدمج" | "concat" => Ok(format!("zaman_concat({})", args_str)),
                    "التكرار" | "repeat" => Ok(format!("zaman_repeat({})", args_str)),
                    "الجذر" | "sqrt" => Ok(format!("zaman_sqrt({})", args_str)),
                    "القوة" | "pow" => Ok(format!("zaman_pow({})", args_str)),
                    "المطلق" | "abs" => Ok(format!("llabs({})", args_str)),
                    "الوقت" | "time" => Ok("((long long)time(NULL))".to_string()),
                    "العشوائي" | "random" => Ok(format!("zaman_random({})", args_str)),
                    "دقيقة" | "min" => Ok(format!("zaman_min({})", args_str)),
                    "أقصى" | "max" => Ok(format!("zaman_max({})", args_str)),
                    _ => Ok(format!("zaman_{}({})", name, args_str)),
                }
            }
            Expr::Array(items) => {
                let mut parts = Vec::new();
                for item in items {
                    parts.push(self.compile_expr(item)?);
                }
                Ok(format!("(long long[]){{{}}}", parts.join(", ")))
            }
            Expr::Index { array, index } => {
                Ok(format!("{}[{}]", self.compile_expr(array)?, self.compile_expr(index)?))
            }
        }
    }

    fn helper_lib() -> &'static str {
        r#"
// ============ مكتبة زمان القياسية ============
static long long zaman_len(const char *s) { return (long long)strlen(s); }
static long long zaman_sqrt(long long n) { return (long long)sqrt((double)n); }
static long long zaman_pow(long long a, long long b) { return (long long)pow((double)a, (double)b); }
static long long zaman_min(long long a, long long b) { return a < b ? a : b; }
static long long zaman_max(long long a, long long b) { return a > b ? a : b; }
static long long zaman_random(long long min, long long max) {
    return min + rand() % (max - min + 1);
}
static const char *zaman_concat(const char *a, const char *b) {
    static char buf[8192];
    snprintf(buf, sizeof(buf), "%s%s", a, b);
    return buf;
}
static const char *zaman_repeat(const char *s, long long n) {
    static char buf[8192];
    buf[0] = 0;
    for (long long i = 0; i < n && strlen(buf) + strlen(s) < sizeof(buf); i++) {
        strcat(buf, s);
    }
    return buf;
}
// ============================================

"#
    }
}
