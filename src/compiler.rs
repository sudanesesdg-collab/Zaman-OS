use crate::ast::{Expr, Stmt, BinOp};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, PartialEq)]
enum VarType {
    Int,
    Str,
}

pub struct Compiler {
    declared_vars: HashSet<String>,
    var_types: HashMap<String, VarType>,
    array_vars: HashSet<String>,
    functions: HashSet<String>,
    fn_param_types: HashMap<String, Vec<VarType>>,
    indent: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            declared_vars: HashSet::new(),
            var_types: HashMap::new(),
            array_vars: HashSet::new(),
            functions: HashSet::new(),
            fn_param_types: HashMap::new(),
            indent: 0,
        }
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        self.collect_functions(stmts);
        self.collect_arrays(stmts);

        let mut out = String::new();
        out.push_str("// كود مُولَّد تلقائيًا من لغة زمان\n");
        out.push_str("#include <stdio.h>\n");
        out.push_str("#include <stdlib.h>\n");
        out.push_str("#include <string.h>\n");
        out.push_str("#include <math.h>\n");
        out.push_str("#include <time.h>\n\n");

        out.push_str(Self::helper_lib());

        for stmt in stmts {
            if let Stmt::Fn { name, params, body } = stmt {
                out.push_str(&self.compile_function(name, params, body)?);
            }
        }

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

    fn collect_arrays(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::Let { name, value } | Stmt::Var { name, value } => {
                    if matches!(value, Expr::Array(_)) {
                        self.array_vars.insert(name.clone());
                    }
                }
                Stmt::Fn { body, .. } => self.collect_arrays(body),
                Stmt::If { then_body, else_body, .. } => {
                    self.collect_arrays(then_body);
                    self.collect_arrays(else_body);
                }
                Stmt::Loop { body, .. } => self.collect_arrays(body),
                Stmt::While { body, .. } => self.collect_arrays(body),
                _ => {}
            }
        }
    }

    fn infer_param_types(&self, params: &[String], body: &[Stmt]) -> HashMap<String, VarType> {
        let mut types = HashMap::new();
        for p in params {
            types.insert(p.clone(), VarType::Int);
        }
        self.scan_body_for_types(body, &mut types);
        types
    }

    fn scan_body_for_types(&self, body: &[Stmt], types: &mut HashMap<String, VarType>) {
        for stmt in body {
            match stmt {
                Stmt::Let { value, .. } | Stmt::Var { value, .. } | Stmt::Expr(value) => {
                    self.scan_expr_for_types(value, types);
                }
                Stmt::Assign { value, .. } => self.scan_expr_for_types(value, types),
                Stmt::IndexAssign { index, value, .. } => {
                    self.scan_expr_for_types(index, types);
                    self.scan_expr_for_types(value, types);
                }
                Stmt::Return(Some(e)) => self.scan_expr_for_types(e, types),
                Stmt::If { condition, then_body, else_body } => {
                    self.scan_expr_for_types(condition, types);
                    self.scan_body_for_types(then_body, types);
                    self.scan_body_for_types(else_body, types);
                }
                Stmt::Loop { from, to, body, .. } => {
                    self.scan_expr_for_types(from, types);
                    self.scan_expr_for_types(to, types);
                    self.scan_body_for_types(body, types);
                }
                Stmt::While { condition, body } => {
                    self.scan_expr_for_types(condition, types);
                    self.scan_body_for_types(body, types);
                }
                _ => {}
            }
        }
    }

    fn scan_expr_for_types(&self, expr: &Expr, types: &mut HashMap<String, VarType>) {
        match expr {
            Expr::Ident(_) => {}
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::Add) || matches!(op, BinOp::Eq) || matches!(op, BinOp::NotEq) {
                    let lt = self.infer_expr_type(left, types);
                    let rt = self.infer_expr_type(right, types);
                    if lt == VarType::Str || rt == VarType::Str {
                        self.mark_as_str(left, types);
                        self.mark_as_str(right, types);
                    }
                }
                self.scan_expr_for_types(left, types);
                self.scan_expr_for_types(right, types);
            }
            Expr::Call { name, args } => {
                if name == "print" || name == "اطبع" 
                    || name == "الدمج" || name == "concat"
                    || name == "التكرار" || name == "repeat"
                    || name == "الطول" || name == "length"
                    || name == "substr" {
                    for arg in args {
                        self.mark_as_str(arg, types);
                        self.scan_expr_for_types(arg, types);
                    }
                } else {
                    for arg in args {
                        self.scan_expr_for_types(arg, types);
                    }
                }
            }
            Expr::Index { array, index } => {
                self.scan_expr_for_types(array, types);
                self.scan_expr_for_types(index, types);
            }
            Expr::Array(items) => {
                for item in items {
                    self.scan_expr_for_types(item, types);
                }
            }
            _ => {}
        }
    }

    fn mark_as_str(&self, expr: &Expr, types: &mut HashMap<String, VarType>) {
        if let Expr::Ident(name) = expr {
            if types.contains_key(name) {
                types.insert(name.clone(), VarType::Str);
            }
        }
    }

    fn infer_expr_type(&self, expr: &Expr, local_types: &HashMap<String, VarType>) -> VarType {
        match expr {
            Expr::Text(_) => VarType::Str,
            Expr::Number(_) | Expr::Bool(_) => VarType::Int,
            Expr::Ident(name) => {
                if let Some(t) = local_types.get(name) {
                    t.clone()
                } else if let Some(t) = self.var_types.get(name) {
                    t.clone()
                } else {
                    VarType::Int
                }
            }
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::Add) {
                    let lt = self.infer_expr_type(left, local_types);
                    let rt = self.infer_expr_type(right, local_types);
                    if lt == VarType::Str || rt == VarType::Str {
                        return VarType::Str;
                    }
                }
                VarType::Int
            }
            Expr::Call { name, .. } => {
                match name.as_str() {
                    "الدمج" | "concat" | "التكرار" | "repeat" 
                    | "input" | "اقرأ" 
                    | "substr" => VarType::Str,
                    _ => VarType::Int,
                }
            }
            _ => VarType::Int,
        }
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<String, String> {
        let param_types = self.infer_param_types(params, body);

        let types_vec: Vec<VarType> = params.iter()
            .map(|p| param_types.get(p).cloned().unwrap_or(VarType::Int))
            .collect();
        self.fn_param_types.insert(name.to_string(), types_vec);

        let mut out = String::new();
        out.push_str(&format!("long long zaman_{}(", name));
        for (i, p) in params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            let ptype = param_types.get(p).cloned().unwrap_or(VarType::Int);
            let ctype = match ptype {
                VarType::Int => "long long",
                VarType::Str => "const char *",
            };
            out.push_str(&format!("{} {}", ctype, p));
        }
        out.push_str(") {\n");

        let old_types: Vec<(String, Option<VarType>)> = params.iter()
            .map(|p| (p.clone(), self.var_types.insert(p.clone(), param_types.get(p).cloned().unwrap_or(VarType::Int))))
            .collect();

        let old_indent = self.indent;
        self.indent = 1;
        for stmt in body {
            out.push_str(&self.compile_stmt(stmt)?);
        }
        self.indent = old_indent;

        for (p, old) in old_types {
            match old {
                Some(t) => { self.var_types.insert(p, t); }
                None => { self.var_types.remove(&p); }
            }
        }

        out.push_str("    return 0LL;\n");
        out.push_str("}\n\n");
        Ok(out)
    }

    fn ind(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn infer_type(&self, expr: &Expr) -> VarType {
        match expr {
            Expr::Text(_) => VarType::Str,
            Expr::Number(_) | Expr::Bool(_) => VarType::Int,
            Expr::Array(_) => VarType::Int,
            Expr::Ident(name) => {
                self.var_types.get(name).cloned().unwrap_or(VarType::Int)
            }
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::Add) {
                    let lt = self.infer_type(left);
                    let rt = self.infer_type(right);
                    if lt == VarType::Str || rt == VarType::Str {
                        return VarType::Str;
                    }
                }
                VarType::Int
            }
            Expr::Call { name, .. } => {
                match name.as_str() {
                    "الدمج" | "concat" | "التكرار" | "repeat" 
                    | "input" | "اقرأ"
                    | "substr" => VarType::Str,
                    _ => VarType::Int,
                }
            }
            Expr::Index { .. } => VarType::Int,
        }
    }

    fn c_type(&self, t: &VarType) -> &'static str {
        match t {
            VarType::Int => "long long",
            VarType::Str => "const char *",
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<String, String> {
        let indent = self.ind();
        match stmt {
            Stmt::Import { .. } => Ok(String::new()),
            Stmt::Let { name, value } | Stmt::Var { name, value } => {
                let vtype = self.infer_type(value);
                self.var_types.insert(name.clone(), vtype.clone());
                
                let mut out = String::new();
                
                if let Expr::Array(items) = value {
                    if !self.declared_vars.contains(name) {
                        self.array_vars.insert(name.clone());
                        let len = items.len();
                        out.push_str(&format!("{}long long {}[{}] = {{", indent, name, len));
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 { out.push_str(", "); }
                            out.push_str(&self.compile_expr(item)?);
                        }
                        out.push_str("};\n");
                        out.push_str(&format!("{}long long {}_len = {}LL;\n", indent, name, len));
                        self.declared_vars.insert(name.clone());
                    }
                    return Ok(out);
                }
                
                if !self.declared_vars.contains(name) {
                    out.push_str(&format!("{}{} {} = ", indent, self.c_type(&vtype), name));
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
                self.var_types.insert(var.clone(), VarType::Int);
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
                    None => Ok(format!("{}return 0LL;\n", indent)),
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
            Expr::Number(n) => Ok(format!("{}LL", n)),
            Expr::Text(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                Ok(format!("\"{}\"", escaped))
            }
            Expr::Bool(b) => Ok(if *b { "1LL".to_string() } else { "0LL".to_string() }),
            Expr::Ident(name) => Ok(name.clone()),
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::Eq) || matches!(op, BinOp::NotEq) {
                    let lt = self.infer_type(left);
                    let rt = self.infer_type(right);
                    if lt == VarType::Str || rt == VarType::Str {
                        let l = self.compile_expr(left)?;
                        let r = self.compile_expr(right)?;
                        let cmp = if matches!(op, BinOp::Eq) { "== 0" } else { "!= 0" };
                        return Ok(format!("(strcmp({}, {}) {})", l, r, cmp));
                    }
                }
                
                if matches!(op, BinOp::Add) {
                    let lt = self.infer_type(left);
                    let rt = self.infer_type(right);
                    if lt == VarType::Str || rt == VarType::Str {
                        let l = self.compile_expr(left)?;
                        let r = self.compile_expr(right)?;
                        if lt == VarType::Str && rt == VarType::Str {
                            return Ok(format!("zaman_concat({}, {})", l, r));
                        } else if lt == VarType::Str {
                            return Ok(format!("zaman_concat_str_int({}, {})", l, r));
                        } else {
                            return Ok(format!("zaman_concat_int_str({}, {})", l, r));
                        }
                    }
                }
                
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
                    let mut fmt = String::new();
                    let mut fmt_args = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { fmt.push(' '); }
                        let atype = self.infer_type(arg);
                        let compiled = self.compile_expr(arg)?;
                        if atype == VarType::Str {
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

                if name == "input" || name == "اقرأ" {
                    return Ok("zaman_input()".to_string());
                }

                if (name == "الطول" || name == "length") && args.len() == 1 {
                    if let Expr::Ident(arr_name) = &args[0] {
                        if self.array_vars.contains(arr_name) {
                            return Ok(format!("{}_len", arr_name));
                        }
                    }
                }

                let mut compiled_args = Vec::new();
                let param_types_opt = self.fn_param_types.get(name).cloned();
                for (i, arg) in args.iter().enumerate() {
                    let compiled = self.compile_expr(arg)?;
                    if let Some(ref ptypes) = param_types_opt {
                        if i < ptypes.len() && ptypes[i] == VarType::Str {
                            let arg_type = self.infer_type(arg);
                            if arg_type == VarType::Int {
                                compiled_args.push(format!("zaman_to_str({})", compiled));
                                continue;
                            }
                        }
                    }
                    compiled_args.push(compiled);
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
                    "substr" => Ok(format!("zaman_substr({})", args_str)),
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
static const char *zaman_concat_str_int(const char *a, long long b) {
    static char buf[8192];
    snprintf(buf, sizeof(buf), "%s%lld", a, b);
    return buf;
}
static const char *zaman_concat_int_str(long long a, const char *b) {
    static char buf[8192];
    snprintf(buf, sizeof(buf), "%lld%s", a, b);
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
static const char *zaman_to_str(long long n) {
    static char buf[64];
    snprintf(buf, sizeof(buf), "%lld", n);
    return buf;
}
static char zaman_input_buf[1024];
static const char *zaman_input(void) {
    if (fgets(zaman_input_buf, sizeof(zaman_input_buf), stdin) == NULL) {
        zaman_input_buf[0] = 0;
        return zaman_input_buf;
    }
    long long len = strlen(zaman_input_buf);
    while (len > 0 && (zaman_input_buf[len-1] == '\n' || zaman_input_buf[len-1] == '\r')) {
        zaman_input_buf[--len] = 0;
    }
    return zaman_input_buf;
}
static const char *zaman_substr(const char *s, long long start, long long end) {
    static char buf[1024];
    long long len = strlen(s);
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start >= end) { buf[0] = 0; return buf; }
    long long n = end - start;
    if (n >= (long long)sizeof(buf)) n = sizeof(buf) - 1;
    for (long long i = 0; i < n; i++) buf[i] = s[start + i];
    buf[n] = 0;
    return buf;
}
// ============================================

"#
    }
}
