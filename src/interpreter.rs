use crate::ast::{Expr, Stmt, BinOp};
use crate::value::{Value, Env};
use std::collections::HashSet;

pub struct Interpreter {
    pub env: Env,
    pub is_mutable: HashSet<String>,
}

enum Flow {
    Normal,
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
            is_mutable: HashSet::new(),
        }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            match self.exec(stmt)? {
                Flow::Return(_) => break,
                Flow::Normal => {}
            }
        }
        Ok(())
    }

    fn exec(&mut self, stmt: &Stmt) -> Result<Flow, String> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.eval(value)?;
                self.env.insert(name.clone(), val);
                Ok(Flow::Normal)
            }
            Stmt::Var { name, value } => {
                let val = self.eval(value)?;
                self.env.insert(name.clone(), val);
                self.is_mutable.insert(name.clone());
                Ok(Flow::Normal)
            }
            Stmt::Assign { name, value } => {
                if !self.env.contains_key(name) {
                    return Err(format!("متغير غير معرّف: {}", name));
                }
                if !self.is_mutable.contains(name) {
                    return Err(format!("لا يمكن تعديل ثابت: {} (استخدم var)", name));
                }
                let val = self.eval(value)?;
                self.env.insert(name.clone(), val);
                Ok(Flow::Normal)
            }
            Stmt::IndexAssign { array, index, value } => {
                if !self.is_mutable.contains(array) {
                    return Err(format!("لا يمكن تعديل مصفوفة ثابتة: {}", array));
                }
                let idx = self.eval(index)?.as_number()? as usize;
                let val = self.eval(value)?;
                
                let mut arr = match self.env.get(array) {
                    Some(Value::Array(a)) => a.clone(),
                    _ => return Err(format!("{} ليس مصفوفة", array)),
                };
                
                if idx >= arr.len() {
                    return Err(format!("فهرس خارج النطاق: {}", idx));
                }
                arr[idx] = val;
                self.env.insert(array.clone(), Value::Array(arr));
                Ok(Flow::Normal)
            }
            Stmt::Fn { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.insert(name.clone(), func);
                Ok(Flow::Normal)
            }
            Stmt::If { condition, then_body, else_body } => {
                let cond = self.eval(condition)?;
                let body = if cond.is_truthy() { then_body } else { else_body };
                for s in body {
                    match self.exec(s)? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal => {}
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::Loop { var, from, to, body } => {
                let start = self.eval(from)?.as_number()?;
                let end = self.eval(to)?.as_number()?;
                let old_val = self.env.get(var).cloned();
                let was_mutable = self.is_mutable.contains(var);
                
                for i in start..=end {
                    self.env.insert(var.clone(), Value::Number(i));
                    self.is_mutable.insert(var.clone());
                    for s in body {
                        match self.exec(s)? {
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal => {}
                        }
                    }
                }
                
                match old_val {
                    Some(v) => { self.env.insert(var.clone(), v); }
                    None => { self.env.remove(var); }
                }
                if !was_mutable { self.is_mutable.remove(var); }
                Ok(Flow::Normal)
            }
            Stmt::While { condition, body } => {
                let mut counter = 0;
                while self.eval(condition)?.is_truthy() {
                    counter += 1;
                    if counter > 1_000_000 {
                        return Err("حلقة لا نهائية".to_string());
                    }
                    for s in body {
                        match self.exec(s)? {
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            Flow::Normal => {}
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => self.eval(e)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(val))
            }
            Stmt::Expr(expr) => {
                self.eval(expr)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Ident(name) => {
                self.env.get(name).cloned()
                    .ok_or_else(|| format!("متغير غير معرّف: {}", name))
            }
            Expr::Array(items) => {
                let mut arr = Vec::new();
                for item in items {
                    arr.push(self.eval(item)?);
                }
                Ok(Value::Array(arr))
            }
            Expr::Index { array, index } => {
                let arr = self.eval(array)?.as_array()?;
                let idx = self.eval(index)?.as_number()? as usize;
                if idx >= arr.len() {
                    return Err(format!("فهرس خارج النطاق: {} (الطول: {})", idx, arr.len()));
                }
                Ok(arr[idx].clone())
            }
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::And) {
                    let l = self.eval(left)?;
                    if !l.is_truthy() { return Ok(Value::Bool(false)); }
                    let r = self.eval(right)?;
                    return Ok(Value::Bool(r.is_truthy()));
                }
                if matches!(op, BinOp::Or) {
                    let l = self.eval(left)?;
                    if l.is_truthy() { return Ok(Value::Bool(true)); }
                    let r = self.eval(right)?;
                    return Ok(Value::Bool(r.is_truthy()));
                }

                let l = self.eval(left)?;
                let r = self.eval(right)?;

                match op {
                    BinOp::Eq => return Ok(Value::Bool(self.values_equal(&l, &r))),
                    BinOp::NotEq => return Ok(Value::Bool(!self.values_equal(&l, &r))),
                    _ => {}
                }

                if matches!(op, BinOp::Add) {
                    if let (Value::Text(a), Value::Text(b)) = (&l, &r) {
                        return Ok(Value::Text(format!("{}{}", a, b)));
                    }
                    if let (Value::Text(a), Value::Number(n)) = (&l, &r) {
                        return Ok(Value::Text(format!("{}{}", a, n)));
                    }
                    if let (Value::Number(n), Value::Text(b)) = (&l, &r) {
                        return Ok(Value::Text(format!("{}{}", n, b)));
                    }
                }

                let ln = l.as_number()?;
                let rn = r.as_number()?;

                let result = match op {
                    BinOp::Add => Value::Number(ln + rn),
                    BinOp::Sub => Value::Number(ln - rn),
                    BinOp::Mul => Value::Number(ln * rn),
                    BinOp::Div => {
                        if rn == 0 { return Err("قسمة على صفر!".to_string()); }
                        Value::Number(ln / rn)
                    }
                    BinOp::Lt => Value::Bool(ln < rn),
                    BinOp::Gt => Value::Bool(ln > rn),
                    BinOp::LtEq => Value::Bool(ln <= rn),
                    BinOp::GtEq => Value::Bool(ln >= rn),
                    _ => return Err("عملية غير مدعومة".to_string()),
                };
                Ok(result)
            }
            Expr::Call { name, args } => {
                match name.as_str() {
                    "print" | "اطبع" => {
                        let mut parts = Vec::new();
                        for arg in args {
                            let v = self.eval(arg)?;
                            parts.push(self.value_to_string(&v));
                        }
                        println!("{}", parts.join(" "));
                        return Ok(Value::Null);
                    }
                    "الطول" | "length" => {
                        let v = self.eval(&args[0])?;
                        return match v {
                            Value::Text(s) => Ok(Value::Number(s.chars().count() as i64)),
                            Value::Array(a) => Ok(Value::Number(a.len() as i64)),
                            _ => Err("الطول يتطلب نصًا أو مصفوفة".to_string()),
                        };
                    }
                    "الدمج" | "concat" => {
                        let a = self.eval(&args[0])?;
                        let b = self.eval(&args[1])?;
                        return Ok(Value::Text(format!(
                            "{}{}",
                            self.value_to_string(&a),
                            self.value_to_string(&b)
                        )));
                    }
                    "التكرار" | "repeat" => {
                        let s = match self.eval(&args[0])? {
                            Value::Text(s) => s,
                            _ => return Err("التكرار يتطلب نصًا".to_string()),
                        };
                        let n = self.eval(&args[1])?.as_number()?;
                        return Ok(Value::Text(s.repeat(n.max(0) as usize)));
                    }
                    "الجذر" | "sqrt" => {
                        let n = self.eval(&args[0])?.as_number()?;
                        if n < 0 { return Err("جذر عدد سالب".to_string()); }
                        return Ok(Value::Number((n as f64).sqrt() as i64));
                    }
                    "القوة" | "pow" => {
                        let a = self.eval(&args[0])?.as_number()?;
                        let b = self.eval(&args[1])?.as_number()?;
                        return Ok(Value::Number((a as f64).powi(b as i32) as i64));
                    }
                    "المطلق" | "abs" => {
                        let n = self.eval(&args[0])?.as_number()?;
                        return Ok(Value::Number(n.abs()));
                    }
                    "الوقت" | "time" => {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let t = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        return Ok(Value::Number(t));
                    }
                    "العشوائي" | "random" => {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let min = self.eval(&args[0])?.as_number()?;
                        let max = self.eval(&args[1])?.as_number()?;
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(0);
                        let range = (max - min + 1) as u64;
                        if range == 0 { return Ok(Value::Number(min)); }
                        let val = min + (seed % range) as i64;
                        return Ok(Value::Number(val));
                    }
                    "دقيقة" | "min" => {
                        let a = self.eval(&args[0])?.as_number()?;
                        let b = self.eval(&args[1])?.as_number()?;
                        return Ok(Value::Number(a.min(b)));
                    }
                    "أقصى" | "max" => {
                        let a = self.eval(&args[0])?.as_number()?;
                        let b = self.eval(&args[1])?.as_number()?;
                        return Ok(Value::Number(a.max(b)));
                    }
                    _ => {}
                }

                let func = self.env.get(name).cloned()
                    .ok_or_else(|| format!("دالة غير معرّفة: {}", name))?;

                if let Value::Function { params, body } = func {
                    if params.len() != args.len() {
                        return Err(format!(
                            "دالة {} تتوقع {} معامل، أُعطي {}",
                            name, params.len(), args.len()
                        ));
                    }

                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval(arg)?);
                    }

                    let old_env = self.env.clone();
                    let old_mut = self.is_mutable.clone();
                    for (param, val) in params.iter().zip(arg_values) {
                        self.env.insert(param.clone(), val);
                        self.is_mutable.insert(param.clone());
                    }

                    let mut result = Value::Null;
                    for stmt in &body {
                        match self.exec(stmt)? {
                            Flow::Return(v) => { result = v; break; }
                            Flow::Normal => {}
                        }
                    }

                    self.env = old_env;
                    self.is_mutable = old_mut;
                    Ok(result)
                } else {
                    Err(format!("{} ليس دالة", name))
                }
            }
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::Text(x), Value::Text(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(a, b)| self.values_equal(a, b))
            }
            _ => false,
        }
    }

    fn value_to_string(&self, v: &Value) -> String {
        match v {
            Value::Number(n) => n.to_string(),
            Value::Text(s) => s.clone(),
            Value::Bool(b) => if *b { "صحيح".to_string() } else { "خطأ".to_string() },
            Value::Null => "عدم".to_string(),
            Value::Function { .. } => "<دالة>".to_string(),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}
