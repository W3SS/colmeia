//! F17 — Starlark hermético (subset) para packs de políticas.
//!
//! Subset suportado: ints, strings, bools, listas, dicts, `def`, `return`,
//! `if`/`else`, atribuição, indexação, chamadas, operadores aritméticos e
//! comparativos, `and`/`or`/`not`. Builtin `rule(regex=, severity=, message=)`
//! e `load()` restrito ao diretório do pack. Sem I/O além do `load`, sem
//! rede, com limite de "gas" (passos de avaliação) contra laços infinitos.
//!
//! Uso típico de um pack:
//! ```python
//! def nivel(path):
//!     if path.endswith(".lock"):
//!         return "block"
//!     return "warn"
//!
//! regras = [
//!     rule(regex="AKIA[0-9A-Z]{16}", severity="critical", message="chave AWS"),
//!     rule(regex="ghp_[A-Za-z0-9]{20,}", severity=nivel("x"), message="token GitHub"),
//! ]
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const GAS_MAX: u64 = 200_000;

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
    Func(Rc<FuncDef>),
    Builtin(&'static str),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Dict(a), Value::Dict(b)) => a == b,
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            (Value::Builtin(a), Value::Builtin(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct FuncDef {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: String,
    pub regex: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug)]
pub struct StarlarkError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for StarlarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "starlark: {} (linha {})", self.message, self.line)
    }
}
impl std::error::Error for StarlarkError {}

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, StarlarkError> {
    Err(StarlarkError {
        message: msg.into(),
        line,
    })
}

// ── Lexer ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Int(i64),
    Str(String),
    Ident(String),
    Punct(&'static str),
    Newline,
    Indent,
    Dedent,
    Eof,
}

fn lex(src: &str) -> Result<Vec<Tok>, StarlarkError> {
    let mut toks = Vec::new();
    let mut indent_stack = vec![0usize];
    let mut cur_indent = 0usize;
    let mut at_line_start = true;

    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        if at_line_start {
            if c == ' ' {
                cur_indent += 1;
                continue;
            }
            if c == '\t' {
                return err(0, "tabulação não é permitida para indentação");
            }
            if c == '\n' {
                cur_indent = 0;
                continue;
            }
            if c == '#' {
                while let Some(&c2) = it.peek() {
                    if c2 == '\n' {
                        break;
                    }
                    it.next();
                }
                continue;
            }
            // fim da indentação desta linha
            let top = *indent_stack.last().unwrap();
            if cur_indent > top {
                indent_stack.push(cur_indent);
                toks.push(Tok::Indent);
            } else if cur_indent < top {
                while *indent_stack.last().unwrap() > cur_indent {
                    indent_stack.pop();
                    toks.push(Tok::Dedent);
                }
                if *indent_stack.last().unwrap() != cur_indent {
                    return err(0, "indentação inconsistente");
                }
            }
            at_line_start = false;
        }

        match c {
            '\n' => {
                toks.push(Tok::Newline);
                at_line_start = true;
                cur_indent = 0;
            }
            ' ' => {}
            '#' => {
                while let Some(&c2) = it.peek() {
                    if c2 == '\n' {
                        break;
                    }
                    it.next();
                }
            }
            '"' | '\'' => {
                let mut s = String::new();
                loop {
                    match it.next() {
                        None => return err(0, "string não terminada"),
                        Some('\\') => match it.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                            None => return err(0, "escape incompleto"),
                        },
                        Some(q) if q == c => break,
                        Some(other) => s.push(other),
                    }
                }
                toks.push(Tok::Str(s));
            }
            c if c.is_ascii_digit() => {
                let mut n = c.to_string();
                while let Some(&c2) = it.peek() {
                    if c2.is_ascii_digit() {
                        n.push(it.next().unwrap());
                    } else {
                        break;
                    }
                }
                toks.push(Tok::Int(n.parse().map_err(|_| StarlarkError {
                    message: "inteiro inválido".into(),
                    line: 0,
                })?));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut s = c.to_string();
                while let Some(&c2) = it.peek() {
                    if c2.is_alphanumeric() || c2 == '_' {
                        s.push(it.next().unwrap());
                    } else {
                        break;
                    }
                }
                toks.push(Tok::Ident(s));
            }
            '+' | '-' | '*' | '%' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | '.' => {
                let p: &'static str = match c {
                    '+' => "+",
                    '-' => "-",
                    '*' => "*",
                    '%' => "%",
                    '(' => "(",
                    ')' => ")",
                    '[' => "[",
                    ']' => "]",
                    '{' => "{",
                    '}' => "}",
                    ',' => ",",
                    ':' => ":",
                    '.' => ".",
                    _ => unreachable!(),
                };
                toks.push(Tok::Punct(p));
            }
            '/' => {
                if it.peek() == Some(&'/') {
                    it.next();
                    toks.push(Tok::Punct("//"));
                } else {
                    return err(0, "'/' sozinho não é suportado; use '//'");
                }
            }
            '=' => {
                if it.peek() == Some(&'=') {
                    it.next();
                    toks.push(Tok::Punct("=="));
                } else {
                    toks.push(Tok::Punct("="));
                }
            }
            '!' => {
                if it.peek() == Some(&'=') {
                    it.next();
                    toks.push(Tok::Punct("!="));
                } else {
                    return err(0, "'!' só pode aparecer em '!=' (use `not`)");
                }
            }
            '<' => {
                if it.peek() == Some(&'=') {
                    it.next();
                    toks.push(Tok::Punct("<="));
                } else {
                    toks.push(Tok::Punct("<"));
                }
            }
            '>' => {
                if it.peek() == Some(&'=') {
                    it.next();
                    toks.push(Tok::Punct(">="));
                } else {
                    toks.push(Tok::Punct(">"));
                }
            }
            other => return err(0, format!("caractere inesperado '{other}'")),
        }
    }
    if !at_line_start {
        toks.push(Tok::Newline);
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        toks.push(Tok::Dedent);
    }
    toks.push(Tok::Eof);
    Ok(toks)
}

// ── AST ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Str(String),
    Bool(bool),
    Ident(String),
    List(Vec<Expr>),
    Dict(Vec<(String, Expr)>),
    Call(Box<Expr>, Vec<(Option<String>, Expr)>),
    Index(Box<Expr>, Box<Expr>),
    Attr(Box<Expr>, String),
    Bin(&'static str, Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    Expr(Expr),
    Assign(String, Expr),
    Def {
        name: String,
        params: Vec<String>,
        body: Rc<Vec<Stmt>>,
    },
    If {
        cond: Expr,
        then: Rc<Vec<Stmt>>,
        els: Rc<Vec<Stmt>>,
    },
    Return(Option<Expr>),
    Load {
        module: String,
        syms: Vec<String>,
    },
}

// ── Parser ─────────────────────────────────────────────────────────────────

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }
    fn next(&mut self) -> Tok {
        let t = self.toks[self.pos.min(self.toks.len() - 1)].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, p: &'static str) -> bool {
        if *self.peek() == Tok::Punct(p) {
            self.next();
            true
        } else {
            false
        }
    }
    fn expect_punct(&mut self, p: &'static str) -> Result<(), StarlarkError> {
        if self.eat(p) {
            Ok(())
        } else {
            err(0, format!("esperado '{p}', encontrado {:?}", self.peek()))
        }
    }
    fn skip_newlines(&mut self) {
        while *self.peek() == Tok::Newline {
            self.next();
        }
    }
    /// como skip_newlines, mas também ignora Indent/Dedent (dentro de
    /// colchetes a indentação é apenas cosmética)
    fn skip_separators(&mut self) {
        while matches!(self.peek(), Tok::Newline | Tok::Indent | Tok::Dedent) {
            self.next();
        }
    }
    fn expect_newline(&mut self) -> Result<(), StarlarkError> {
        if *self.peek() == Tok::Newline {
            self.next();
            Ok(())
        } else if *self.peek() == Tok::Eof || *self.peek() == Tok::Dedent {
            Ok(())
        } else {
            err(0, format!("esperada quebra de linha, encontrado {:?}", self.peek()))
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, StarlarkError> {
        // após ':' — mesmo linha simples ou bloco indentado
        self.skip_newlines();
        if self.eat("{") {
            let mut stmts = Vec::new();
            loop {
                self.skip_newlines();
                if self.eat("}") {
                    break;
                }
                stmts.push(self.stmt()?);
            }
            Ok(stmts)
        } else if *self.peek() == Tok::Indent {
            self.next();
            let mut stmts = Vec::new();
            loop {
                self.skip_newlines();
                if *self.peek() == Tok::Dedent {
                    self.next();
                    break;
                }
                if *self.peek() == Tok::Eof {
                    break;
                }
                stmts.push(self.stmt()?);
            }
            Ok(stmts)
        } else {
            // statement simples na mesma linha
            Ok(vec![self.stmt()?])
        }
    }

    fn stmt(&mut self) -> Result<Stmt, StarlarkError> {
        match self.next() {
            Tok::Ident(id) => match id.as_str() {
                "def" => {
                    let name = match self.next() {
                        Tok::Ident(n) => n,
                        t => return err(0, format!("esperado nome de função, encontrado {t:?}")),
                    };
                    self.expect_punct("(")?;
                    let mut params = Vec::new();
                    if !self.eat(")") {
                        loop {
                            match self.next() {
                                Tok::Ident(p) => params.push(p),
                                t => return err(0, format!("parâmetro inválido: {t:?}")),
                            }
                            if self.eat(")") {
                                break;
                            }
                            self.expect_punct(",")?;
                        }
                    }
                    self.expect_punct(":")?;
                    let body = Rc::new(self.block()?);
                    Ok(Stmt::Def { name, params, body })
                }
                "if" => {
                    let cond = self.expr()?;
                    self.expect_punct(":")?;
                    let then = Rc::new(self.block()?);
                    self.skip_newlines();
                    let els = if *self.peek() == Tok::Ident("else".to_string()) {
                        self.next();
                        self.expect_punct(":")?;
                        Rc::new(self.block()?)
                    } else {
                        Rc::new(Vec::new())
                    };
                    Ok(Stmt::If { cond, then, els })
                }
                "return" => {
                    if *self.peek() == Tok::Newline {
                        self.expect_newline()?;
                        Ok(Stmt::Return(None))
                    } else {
                        let e = self.expr()?;
                        self.expect_newline()?;
                        Ok(Stmt::Return(Some(e)))
                    }
                }
                "load" => {
                    self.expect_punct("(")?;
                    let module = match self.next() {
                        Tok::Str(m) => m,
                        t => return err(0, format!("load: módulo deve ser string, {t:?}")),
                    };
                    let mut syms = Vec::new();
                    while self.eat(",") {
                        match self.next() {
                            Tok::Str(s) => syms.push(s),
                            t => return err(0, format!("load: símbolo deve ser string, {t:?}")),
                        }
                    }
                    self.expect_punct(")")?;
                    self.expect_newline()?;
                    Ok(Stmt::Load { module, syms })
                }
                _ => self.stmt_with_ident(id),
            },
            t => err(0, format!("statement inesperado: {t:?}")),
        }
    }

    fn stmt_with_ident(&mut self, id: String) -> Result<Stmt, StarlarkError> {
        // já consumiu um Ident — pode ser atribuição ou expressão começando com ident
        if *self.peek() == Tok::Punct("=") {
            self.next();
            let e = self.expr()?;
            self.expect_newline()?;
            return Ok(Stmt::Assign(id, e));
        }
        let e = self.continue_expr(Expr::Ident(id))?;
        self.expect_newline()?;
        Ok(Stmt::Expr(e))
    }

    // ── expressões (precedência crescente) ─────────────────────────────

    fn expr(&mut self) -> Result<Expr, StarlarkError> {
        let t = self.next();
        let atom = match t {
            Tok::Int(n) => Expr::Int(n),
            Tok::Str(s) => Expr::Str(s),
            Tok::Ident(id) => match id.as_str() {
                "True" => Expr::Bool(true),
                "False" => Expr::Bool(false),
                "not" => {
                    let e = self.expr()?;
                    return Ok(Expr::Not(Box::new(e)));
                }
                _ => Expr::Ident(id),
            },
            Tok::Punct("(") => {
                let e = self.expr()?;
                self.expect_punct(")")?;
                e
            }
            Tok::Punct("[") => {
                let mut items = Vec::new();
                self.skip_separators();
                if self.eat("]") {
                    return Ok(Expr::List(items));
                }
                loop {
                    items.push(self.expr()?);
                    self.skip_separators();
                    if self.eat("]") {
                        break;
                    }
                    self.expect_punct(",")?;
                    self.skip_separators();
                    if self.eat("]") {
                        break;
                    }
                }
                Expr::List(items)
            }
            Tok::Punct("{") => {
                let mut pairs = Vec::new();
                self.skip_separators();
                if self.eat("}") {
                    return Ok(Expr::Dict(pairs));
                }
                loop {
                    let key = match self.next() {
                        Tok::Str(s) => s,
                        Tok::Ident(i) => i,
                        t => return err(0, format!("chave de dict inválida: {t:?}")),
                    };
                    self.expect_punct(":")?;
                    let v = self.expr()?;
                    pairs.push((key, v));
                    self.skip_separators();
                    if self.eat("}") {
                        break;
                    }
                    self.expect_punct(",")?;
                    self.skip_separators();
                    if self.eat("}") {
                        break;
                    }
                }
                Expr::Dict(pairs)
            }
            t => return err(0, format!("expressão inesperada: {t:?}")),
        };
        self.continue_expr(atom)
    }

    /// continua uma expressão primária com pós-fixos e binários
    fn continue_expr(&mut self, mut lhs: Expr) -> Result<Expr, StarlarkError> {
        loop {
            match self.peek().clone() {
                Tok::Punct("(") => {
                    self.next();
                    let mut args = Vec::new();
                    self.skip_separators();
                    if !self.eat(")") {
                        loop {
                            // argumento nomeado?
                            if let Tok::Ident(name) = self.peek().clone() {
                                if self.toks.get(self.pos + 1) == Some(&Tok::Punct("=")) {
                                    self.next();
                                    self.next();
                                    args.push((Some(name), self.expr()?));
                                } else {
                                    args.push((None, self.expr()?));
                                }
                            } else {
                                args.push((None, self.expr()?));
                            }
                            self.skip_separators();
                            if self.eat(")") {
                                break;
                            }
                            self.expect_punct(",")?;
                            self.skip_separators();
                            if self.eat(")") {
                                break;
                            }
                        }
                    }
                    lhs = Expr::Call(Box::new(lhs), args);
                }
                Tok::Punct("[") => {
                    self.next();
                    let idx = self.expr()?;
                    self.expect_punct("]")?;
                    lhs = Expr::Index(Box::new(lhs), Box::new(idx));
                }
                Tok::Punct(".") => {
                    self.next();
                    match self.next() {
                        Tok::Ident(attr) => lhs = Expr::Attr(Box::new(lhs), attr),
                        t => return err(0, format!("atributo inválido: {t:?}")),
                    }
                }
                Tok::Punct(op @ ("+" | "-" | "*" | "//" | "%")) => {
                    self.next();
                    let rhs = self.arith()?;
                    lhs = Expr::Bin(leak(op), Box::new(lhs), Box::new(rhs));
                }
                Tok::Punct(op @ ("==" | "!=" | "<" | "<=" | ">" | ">=")) => {
                    self.next();
                    let rhs = self.arith()?;
                    lhs = Expr::Bin(leak(op), Box::new(lhs), Box::new(rhs));
                }
                Tok::Ident(id) if id == "and" => {
                    self.next();
                    let rhs = self.arith()?;
                    lhs = Expr::And(Box::new(lhs), Box::new(rhs));
                }
                Tok::Ident(id) if id == "or" => {
                    self.next();
                    let rhs = self.arith()?;
                    lhs = Expr::Or(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn arith(&mut self) -> Result<Expr, StarlarkError> {
        let t = self.next();
        let mut lhs = match t {
            Tok::Int(n) => Expr::Int(n),
            Tok::Str(s) => Expr::Str(s),
            Tok::Ident(id) => Expr::Ident(id),
            Tok::Punct("(") => {
                let e = self.expr()?;
                self.expect_punct(")")?;
                e
            }
            Tok::Punct("-") => {
                let e = self.arith()?;
                match e {
                    Expr::Int(n) => Expr::Int(-n),
                    other => Expr::Bin("-", Box::new(Expr::Int(0)), Box::new(other)),
                }
            }
            other => return err(0, format!("operando inválido: {other:?}")),
        };
        loop {
            match self.peek() {
                Tok::Punct(op @ ("+" | "-" | "*" | "//" | "%")) => {
                    let op = *op;
                    self.next();
                    let rhs = self.arith()?;
                    lhs = Expr::Bin(leak(op), Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}

fn leak(s: &str) -> &'static str {
    match s {
        "+" => "+",
        "-" => "-",
        "*" => "*",
        "//" => "//",
        "%" => "%",
        "==" => "==",
        "!=" => "!=",
        "<" => "<",
        "<=" => "<=",
        ">" => ">",
        ">=" => ">=",
        _ => unreachable!(),
    }
}

// ── Avaliador ──────────────────────────────────────────────────────────────

struct Env {
    scopes: Vec<BTreeMap<String, Value>>,
    gas: u64,
    pack_dir: PathBuf,
}

impl Env {
    fn lookup(&self, name: &str) -> Option<Value> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Some(v.clone());
            }
        }
        None
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Str(s) => !s.is_empty(),
        Value::List(l) => !l.is_empty(),
        Value::Dict(d) => !d.is_empty(),
        Value::Func(_) | Value::Builtin(_) => true,
    }
}

fn eval(e: &Expr, env: &mut Env) -> Result<Value, StarlarkError> {
    env.gas = env.gas.saturating_sub(1);
    if env.gas == 0 {
        return err(0, "limite de execução excedido (gas esgotado)");
    }
    match e {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => env
            .lookup(name)
            .ok_or_else(|| StarlarkError {
                message: format!("símbolo '{name}' não definido"),
                line: 0,
            }),
        Expr::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for i in items {
                out.push(eval(i, env)?);
            }
            Ok(Value::List(out))
        }
        Expr::Dict(pairs) => {
            let mut out = BTreeMap::new();
            for (k, v) in pairs {
                out.insert(k.clone(), eval(v, env)?);
            }
            Ok(Value::Dict(out))
        }
        Expr::Not(inner) => Ok(Value::Bool(!truthy(&eval(inner, env)?))),
        Expr::And(a, b) => {
            let av = eval(a, env)?;
            if !truthy(&av) {
                Ok(av)
            } else {
                eval(b, env)
            }
        }
        Expr::Or(a, b) => {
            let av = eval(a, env)?;
            if truthy(&av) {
                Ok(av)
            } else {
                eval(b, env)
            }
        }
        Expr::Bin(op, a, b) => {
            let av = eval(a, env)?;
            let bv = eval(b, env)?;
            binop(op, &av, &bv)
        }
        Expr::Index(base, idx) => {
            let b = eval(base, env)?;
            let i = eval(idx, env)?;
            match (&b, &i) {
                (Value::List(l), Value::Int(n)) => {
                    let idx = usize::try_from(*n)
                        .ok()
                        .filter(|x| *x < l.len())
                        .ok_or_else(|| StarlarkError {
                            message: format!("índice {n} fora da lista (len {})", l.len()),
                            line: 0,
                        })?;
                    Ok(l[idx].clone())
                }
                (Value::Dict(d), Value::Str(k)) => Ok(d.get(k).cloned().unwrap_or(Value::Bool(false))),
                (Value::Str(s), Value::Int(n)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let idx = usize::try_from(*n)
                        .ok()
                        .filter(|x| *x < chars.len())
                        .ok_or_else(|| StarlarkError {
                            message: format!("índice {n} fora da string"),
                            line: 0,
                        })?;
                    Ok(Value::Str(chars[idx].to_string()))
                }
                _ => err(0, "indexação não suportada para esses tipos"),
            }
        }
        Expr::Attr(_, _) => err(0, "atributos não são suportados no subset"),
        Expr::Call(f, args) => {
            // métodos de string embutidos: base.endswith(...), etc.
            if let Expr::Attr(base_expr, method) = f.as_ref() {
                if let Value::Str(s) = eval(base_expr, env)? {
                    let str_method: Option<&str> = match method.as_str() {
                            "endswith" | "startswith" | "lower" | "upper" => Some(method),
                            _ => None,
                        };
                    if let Some(m) = str_method {
                        let argv = eval_args(args, env)?;
                        return str_builtin(m, &s, &argv);
                    }
                }
            }
            let fv = eval(f, env)?;
            let mut pos = Vec::new();
            let mut named: BTreeMap<String, Value> = BTreeMap::new();
            for (n, a) in args {
                let v = eval(a, env)?;
                match n {
                    Some(name) => {
                        named.insert(name.clone(), v);
                    }
                    None => pos.push(v),
                }
            }
            match fv {
                Value::Builtin("len") => {
                    if pos.len() != 1 {
                        return err(0, "len() espera 1 argumento");
                    }
                    let n = match &pos[0] {
                        Value::Str(s) => s.chars().count() as i64,
                        Value::List(l) => l.len() as i64,
                        Value::Dict(d) => d.len() as i64,
                        _ => return err(0, "len() não suporta esse tipo"),
                    };
                    Ok(Value::Int(n))
                }
                Value::Func(fd) => {
                    if pos.len() != fd.params.len() {
                        return err(
                            0,
                            format!(
                                "função '{}' espera {} argumentos, recebeu {}",
                                fd.name,
                                fd.params.len(),
                                pos.len()
                            ),
                        );
                    }
                    env.scopes.push(BTreeMap::new());
                    for (p, v) in fd.params.iter().zip(pos) {
                        env.scopes.last_mut().unwrap().insert(p.clone(), v);
                    }
                    let r = exec_block(&fd.body, env);
                    env.scopes.pop();
                    r.map(|v| v.unwrap_or(Value::Bool(false)))
                }
                Value::Dict(d) if d.get("__builtin__") == Some(&Value::Str("rule".into())) => {
                    // chamada ao builtin rule(**kwargs): retorna dict marcado
                    let mut out = named;
                    out.insert("__builtin__".into(), Value::Str("rule_result".into()));
                    Ok(Value::Dict(out))
                }
                _ => err(0, "valor não é chamável"),
            }
        }
    }
}

fn eval_args(args: &[(Option<String>, Expr)], env: &mut Env) -> Result<Vec<Value>, StarlarkError> {
    let mut out = Vec::with_capacity(args.len());
    for (_, a) in args {
        out.push(eval(a, env)?);
    }
    Ok(out)
}

fn str_builtin(method: &str, s: &str, argv: &[Value]) -> Result<Value, StarlarkError> {
    match method {
        "endswith" | "startswith" => {
            if argv.len() != 1 {
                return err(0, format!("{method}() espera 1 argumento"));
            }
            let needle = match &argv[0] {
                Value::Str(n) => n.clone(),
                _ => return err(0, format!("{method}() espera string")),
            };
            let r = if method == "endswith" {
                s.ends_with(&needle)
            } else {
                s.starts_with(&needle)
            };
            Ok(Value::Bool(r))
        }
        "lower" => Ok(Value::Str(s.to_lowercase())),
        "upper" => Ok(Value::Str(s.to_uppercase())),
        _ => err(0, format!("método de string não suportado: {method}")),
    }
}

fn binop(op: &str, a: &Value, b: &Value) -> Result<Value, StarlarkError> {
    match (op, a, b) {
        ("+", Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        ("-", Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
        ("*", Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
        ("//", Value::Int(x), Value::Int(y)) => {
            if *y == 0 {
                err(0, "divisão por zero")
            } else {
                Ok(Value::Int(x / y))
            }
        }
        ("%", Value::Int(x), Value::Int(y)) => {
            if *y == 0 {
                err(0, "módulo por zero")
            } else {
                Ok(Value::Int(x % y))
            }
        }
        ("+", Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{x}{y}"))),
        ("+", Value::List(x), Value::List(y)) => {
            let mut out = x.clone();
            out.extend(y.iter().cloned());
            Ok(Value::List(out))
        }
        ("==", x, y) => Ok(Value::Bool(x == y)),
        ("!=", x, y) => Ok(Value::Bool(x != y)),
        ("<", Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)),
        ("<=", Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)),
        (">", Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)),
        (">=", Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)),
        (other, _, _) => err(0, format!("operação não suportada: {other}")),
    }
}

fn exec_block(stmts: &[Stmt], env: &mut Env) -> Result<Option<Value>, StarlarkError> {
    for s in stmts {
        match exec_stmt(s, env)? {
            Some(v) => return Ok(Some(v)),
            None => {}
        }
    }
    Ok(None)
}

fn exec_stmt(s: &Stmt, env: &mut Env) -> Result<Option<Value>, StarlarkError> {
    env.gas = env.gas.saturating_sub(1);
    if env.gas == 0 {
        return err(0, "limite de execução excedido (gas esgotado)");
    }
    match s {
        Stmt::Expr(e) => {
            eval(e, env)?;
            Ok(None)
        }
        Stmt::Assign(name, e) => {
            let v = eval(e, env)?;
            env.scopes.last_mut().unwrap().insert(name.clone(), v);
            Ok(None)
        }
        Stmt::Def { name, params, body } => {
            env.scopes.last_mut().unwrap().insert(
                name.clone(),
                Value::Func(Rc::new(FuncDef {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.as_ref().clone(),
                })),
            );
            Ok(None)
        }
        Stmt::If { cond, then, els } => {
            if truthy(&eval(cond, env)?) {
                exec_block(then, env)
            } else {
                exec_block(els, env)
            }
        }
        Stmt::Return(e) => match e {
            Some(e) => Ok(Some(eval(e, env)?)),
            None => Ok(Some(Value::Bool(false))),
        },
        Stmt::Load { module, syms } => {
            let path = env
                .pack_dir
                .join(module)
                .canonicalize()
                .map_err(|_| StarlarkError {
                    message: format!("módulo '{module}' não encontrado no pack"),
                    line: 0,
                })?;
            if !path.starts_with(&env.pack_dir) {
                return err(0, "load() fora do diretório do pack não é permitido");
            }
            let src = std::fs::read_to_string(&path).map_err(|e| StarlarkError {
                message: format!("falha ao ler módulo '{module}': {e}"),
                line: 0,
            })?;
            let mut loaded = run_source(&src, &path, GAS_MAX / 2)?;
            for sym in syms {
                let v = loaded
                    .bindings
                    .remove(sym)
                    .ok_or_else(|| StarlarkError {
                        message: format!("símbolo '{sym}' não exportado por '{module}'"),
                        line: 0,
                    })?;
                env.scopes.last_mut().unwrap().insert(sym.clone(), v);
            }
            Ok(None)
        }
    }
}

struct RunOutcome {
    pub rules: Vec<RuleDef>,
    pub bindings: BTreeMap<String, Value>,
}

fn collect_rules(v: &Value, out: &mut Vec<RuleDef>) {
    match v {
        Value::Dict(d) => {
            if d.get("__builtin__") == Some(&Value::Str("rule_result".into())) {
                let get = |k: &str| match d.get(k) {
                    Some(Value::Str(s)) => s.clone(),
                    Some(Value::Int(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    _ => String::new(),
                };
                out.push(RuleDef {
                    id: get("id"),
                    regex: get("regex"),
                    severity: get("severity"),
                    message: get("message"),
                });
            } else {
                for val in d.values() {
                    collect_rules(val, out);
                }
            }
        }
        Value::List(l) => {
            for item in l {
                collect_rules(item, out);
            }
        }
        _ => {}
    }
}

fn run_source(src: &str, file: &Path, gas: u64) -> Result<RunOutcome, StarlarkError> {
    let toks = lex(src)?;
    let mut p = Parser { toks, pos: 0 };
    let mut stmts = Vec::new();
    p.skip_newlines();
    while *p.peek() != Tok::Eof {
        stmts.push(p.stmt()?);
        p.skip_newlines();
    }
    let mut env = Env {
        scopes: vec![BTreeMap::new()],
        gas,
        pack_dir: file
            .parent()
            .map(|x| x.canonicalize().unwrap_or_else(|_| x.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    // builtins
    let mut rule_marker = BTreeMap::new();
    rule_marker.insert("__builtin__".into(), Value::Str("rule".into()));
    env.scopes[0].insert("rule".into(), Value::Dict(rule_marker));
    env.scopes[0].insert("len".into(), Value::Builtin("len"));
    exec_block(&stmts, &mut env)?;
    let bindings = env.scopes.pop().unwrap();
    // convenção: se existir binding `rules`/`regras`, coleta apenas dele;
    // senão varre todos os bindings de topo com dedup.
    let mut rules: Vec<RuleDef> = Vec::new();
    if let Some(v) = bindings
        .get("rules")
        .or_else(|| bindings.get("regras"))
        .or_else(|| bindings.get("REGRAS"))
    {
        collect_rules(v, &mut rules);
    } else {
        for v in bindings.values() {
            collect_rules(v, &mut rules);
        }
    }
    rules.retain(|r| !r.regex.is_empty());
    rules.dedup_by(|a, b| a.regex == b.regex && a.severity == b.severity && a.message == b.message);
    Ok(RunOutcome { rules, bindings })
}

/// Executa um pack a partir do arquivo de entrada e devolve as regras
/// coletadas dos bindings de topo.
pub fn run_pack(entry: &Path) -> Result<Vec<RuleDef>, StarlarkError> {
    let src = std::fs::read_to_string(entry).map_err(|e| StarlarkError {
        message: format!("falha ao ler '{}': {e}", entry.display()),
        line: 0,
    })?;
    Ok(run_source(&src, entry, GAS_MAX)?.rules)
}
