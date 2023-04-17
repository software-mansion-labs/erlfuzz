/* Copyright (c) Meta Platforms, Inc. and affiliates. All rights reserved.
 *
 * This source code is licensed under the Apache 2.0 license found in
 * the LICENSE file in the root directory of this source tree.
 */
use std::fmt;

use num_bigint::BigInt;
use BinaryOperator::*;
use UnaryOperator::*;

use crate::core_types::*;
use crate::Config;

pub trait SizedAst {
    fn size(&self, module: &Module) -> ASTSize;
}

impl<T: SizedAst> SizedAst for Vec<T> {
    fn size(&self, module: &Module) -> ASTSize {
        self.iter().map(|node| node.size(module)).sum::<ASTSize>()
    }
}
impl<T1: SizedAst, T2: SizedAst> SizedAst for (T1, T2) {
    fn size(&self, module: &Module) -> ASTSize {
        self.0.size(module) + self.1.size(module)
    }
}
impl<T1: SizedAst, T2: SizedAst, T3: SizedAst> SizedAst for (T1, T2, T3) {
    fn size(&self, module: &Module) -> ASTSize {
        self.0.size(module) + self.1.size(module) + self.2.size(module)
    }
}

pub trait AstNode: SizedAst {
    fn fmt(&self, module: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<T: SizedAst> SizedAst for Option<T> {
    fn size(&self, module: &Module) -> ASTSize {
        match self {
            None => 0,
            Some(node) => node.size(module),
        }
    }
}
impl<T: AstNode> AstNode for Option<T> {
    fn fmt(&self, module: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            None => Ok(()),
            Some(node) => node.fmt(module, f),
        }
    }
}

struct WithModuleRef<'a, T> {
    module: &'a Module<'a>,
    value: T,
}
impl<'a, T: AstNode> fmt::Display for WithModuleRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        AstNode::fmt(&self.value, self.module, f)
    }
}

fn with_module<'a, T>(value: T, module: &'a Module) -> WithModuleRef<'a, T> {
    WithModuleRef { value, module }
}

fn write_list_strings<I: Iterator<Item = String>>(
    f: &mut fmt::Formatter<'_>,
    l: I,
    separator: &str,
) -> fmt::Result {
    let mut is_first = true;
    for x in l {
        if !is_first {
            write!(f, "{}", separator)?;
        }
        is_first = false;
        write!(f, "{}", x)?;
    }
    Ok(())
}

fn write_list_ast_nodes<T: AstNode + Copy>(
    f: &mut fmt::Formatter<'_>,
    m: &Module,
    l: &Vec<T>,
    separator: &str,
) -> fmt::Result {
    let mut is_first = true;
    for x in l {
        if !is_first {
            write!(f, "{}", separator)?;
        }
        is_first = false;
        write!(f, "{}", with_module(*x, m))?;
    }
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperator {
    Eq,
    NEq,
    LTE,
    LT,
    GTE,
    GT,
    ExactlyEq,
    ExactlyNEq,
    BinaryPlus,
    BinaryMinus,
    Mult,
    Slash,
    Div,
    Rem,
    BAnd,
    BOr,
    BXor,
    BSl,
    BSr,
    And,
    Or,
    Xor,
    AndAlso,
    OrElse,
    PlusPlus,
    MinusMinus,
}
impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Eq => "==",
                NEq => "/=",
                LTE => "=<",
                LT => "<",
                GTE => ">=",
                GT => ">",
                ExactlyEq => "=:=",
                ExactlyNEq => "=/=",
                BinaryPlus => "+",
                BinaryMinus => "-",
                Mult => "*",
                Slash => "/",
                Div => "div",
                Rem => "rem",
                BAnd => "band",
                BOr => "bor",
                BXor => "bxor",
                BSl => "bsl",
                BSr => "bsr",
                And => "and",
                Or => "or",
                Xor => "xor",
                AndAlso => "andalso",
                OrElse => "orelse",
                PlusPlus => "++",
                MinusMinus => "--",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOperator {
    UnaryPlus,
    UnaryMinus,
    BooleanNot,
    BitwiseNot,
}
impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryPlus => "+",
                UnaryMinus => "-",
                BooleanNot => "not",
                BitwiseNot => "bnot",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Var(VarNum),
    Nil(),
    Atom(Atom),
    Integer(BigInt),
    Float(f64),
    String(String),
    LocalCall(String, Vec<ExprId>),
    RemoteCall(String, String, Vec<ExprId>),
    Tuple(Vec<ExprId>),
    Catch(ExprId),
    BinaryOperation(BinaryOperator, ExprId, ExprId),
    UnaryOperation(UnaryOperator, ExprId),
    Case(ExprId, Vec<(PatternId, GuardSeqId, BodyId)>),
    Assignment(PatternId, ExprId),
    MapLiteral(Vec<(ExprId, ExprId)>),
    MapInsertion(ExprId, ExprId, ExprId),
    MapUpdate(ExprId, ExprId, ExprId),
    BitstringConstruction(Vec<(ExprId, Option<ExprId>, TypeSpecifier)>),
    Fun(Option<VarNum>, Vec<FunctionClauseId>),
    Comprehension(ComprehensionKind, ExprId, Vec<ComprehensionElement>),
    MapComprehension(ExprId, ExprId, Vec<ComprehensionElement>),
    Try(
        BodyId,                                       // Exprs
        Option<Vec<(PatternId, GuardSeqId, BodyId)>>, // "of" section
        Option<Vec<(PatternId, GuardSeqId, BodyId)>>, // "catch" section
        Option<BodyId>,                               // "after" section
    ),
    Maybe(
        Vec<MaybeExpr>,
        Option<Vec<(PatternId, GuardSeqId, BodyId)>>, // "else" section
    ),
    Block(BodyId),
}
impl SizedAst for Expr {
    fn size(&self, module: &Module) -> ASTSize {
        match self {
            Expr::LocalCall(_, args) | Expr::RemoteCall(_, _, args) => 1 + args.size(module),
            Expr::Tuple(elements) => 1 + elements.size(module),
            Expr::Catch(e) => 1 + e.size(module),
            Expr::BinaryOperation(_, e1, e2) => 1 + e1.size(module) + e2.size(module),
            Expr::UnaryOperation(_, e) => e.size(module),
            Expr::Case(e, cases) => 1 + e.size(module) + cases.size(module),
            Expr::Assignment(p, e) => 1 + p.size(module) + e.size(module),
            Expr::MapLiteral(mappings) => 1 + mappings.size(module),
            Expr::MapInsertion(map, k, v) | Expr::MapUpdate(map, k, v) => {
                1 + map.size(module) + k.size(module) + v.size(module)
            }
            Expr::BitstringConstruction(elements) => 1 + elements.size(module),
            Expr::Comprehension(_kind, head, elements) => {
                1 + head.size(module) + elements.size(module)
            }
            Expr::MapComprehension(head_key, head_value, elements) => {
                1 + head_key.size(module) + head_value.size(module) + elements.size(module)
            }
            Expr::Fun(_, clauses) => 1 + clauses.size(module),
            Expr::Try(exprs, of, catch, after) => {
                1 + exprs.size(module) + of.size(module) + catch.size(module) + after.size(module)
            }
            Expr::Maybe(exprs, else_section) => 1 + exprs.size(module) + else_section.size(module),
            Expr::Block(b) => 1 + b.size(module),
            Expr::Var(_)
            | Expr::Nil()
            | Expr::Atom(_)
            | Expr::Integer(_)
            | Expr::Float(_)
            | Expr::String(_) => 1,
        }
    }
}
impl AstNode for Expr {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(v) => write!(f, "_V{}", v),
            Expr::Nil() => write!(f, "[]"),
            Expr::Atom(a) => write!(f, "{}", a),
            Expr::Integer(n) => write!(f, "{}", n),
            Expr::Float(n) => {
                // Force a ".0" at the end if the double is actually an integer
                if n.trunc() != *n {
                    write!(f, "{}", n)
                } else {
                    write!(f, "{:.1}", n)
                }
            }
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::LocalCall(fun_name, args) => {
                write!(f, "({}(", fun_name)?;
                write_list_ast_nodes(f, m, args, ", ")?;
                write!(f, "))")
            }
            Expr::RemoteCall(module_name, fun_name, args) => {
                write!(f, "({}:{}(", module_name, fun_name)?;
                write_list_ast_nodes(f, m, args, ", ")?;
                write!(f, "))")
            }
            Expr::Tuple(es) => {
                // "{{" and "}}" are escaping to show a single "{" and "}" respectively
                write!(f, "{{")?;
                write_list_ast_nodes(f, m, es, ", ")?;
                write!(f, "}}")
            }
            Expr::Catch(e) => write!(f, "(catch {})", with_module(*e, m)),
            Expr::BinaryOperation(op, e1, e2) => {
                write!(
                    f,
                    "({} {} {})",
                    with_module(*e1, m),
                    op,
                    with_module(*e2, m)
                )
            }
            Expr::UnaryOperation(op, e) => write!(f, "({} {})", op, with_module(*e, m)),
            Expr::Case(e, cases) => {
                write!(f, "(case {} of\n\t", with_module(*e, m))?;
                write_list_strings(
                    f,
                    cases.iter().map(|(p, g, b)| {
                        format!(
                            "{} {} ->\n\t{}",
                            with_module(*p, m),
                            with_module(*g, m),
                            with_module(*b, m)
                        )
                    }),
                    ";\n\t",
                )?;
                write!(f, "\n\tend)")
            }
            Expr::Assignment(p, e) => {
                write!(f, "({} = {})", with_module(*p, m), with_module(*e, m))
            }
            Expr::MapLiteral(mappings) => {
                write!(f, "#{{")?;
                write_list_strings(
                    f,
                    mappings
                        .iter()
                        .map(|(k, v)| format!("{} => {}", with_module(*k, m), with_module(*v, m))),
                    ", ",
                )?;
                write!(f, "}}")
            }
            Expr::MapInsertion(map, k, v) => write!(
                f,
                "({} #{{ {} => {} }})",
                with_module(*map, m),
                with_module(*k, m),
                with_module(*v, m)
            ),
            Expr::MapUpdate(map, k, v) => write!(
                f,
                "({} #{{ {} := {} }})",
                with_module(*map, m),
                with_module(*k, m),
                with_module(*v, m)
            ),
            Expr::BitstringConstruction(elements) => {
                write!(f, "(<<")?;
                write_list_strings(
                    f,
                    elements.iter().map(|(value, size, type_specifiers)| {
                        let size = size
                            .map(|s| format!(":({})", with_module(s, m)))
                            .unwrap_or_default();
                        format!("({}){}{}", with_module(*value, m), size, type_specifiers)
                    }),
                    ", ",
                )?;
                write!(f, ">>)")
            }
            Expr::Comprehension(kind, expr, elements) => {
                match kind {
                    ComprehensionKind::List => write!(f, "[")?,
                    ComprehensionKind::Bitstring => write!(f, "<<")?,
                }
                write!(f, " ({}) || ", with_module(*expr, m))?;
                write_list_ast_nodes(f, m, elements, ", ")?;
                match kind {
                    ComprehensionKind::List => write!(f, "]"),
                    ComprehensionKind::Bitstring => write!(f, ">>"),
                }
            }
            Expr::MapComprehension(key, value, elements) => {
                write!(
                    f,
                    // The extra '{' serves as escaping
                    "#{{ {} => {} || ",
                    with_module(*key, m),
                    with_module(*value, m)
                )?;
                write_list_ast_nodes(f, m, elements, ", ")?;
                write!(f, "}}")
            }
            Expr::Fun(_, clauses) => {
                write!(f, "fun ")?;
                write_list_ast_nodes(f, m, clauses, "; ")?;
                write!(f, " end")
            }
            Expr::Try(exprs, of, catch, after) => {
                write!(f, "(try ")?;
                exprs.fmt(m, f)?;
                // FIXME: share all of this redundant code
                if let Some(cases) = of {
                    write!(f, " of\n\t")?;
                    write_list_strings(
                        f,
                        cases.iter().map(|(p, g, b)| {
                            format!(
                                "{} {} ->\n\t{}",
                                with_module(*p, m),
                                with_module(*g, m),
                                with_module(*b, m)
                            )
                        }),
                        ";\n\t",
                    )?;
                }
                if let Some(cases) = catch {
                    write!(f, "\ncatch\n\t")?;
                    write_list_strings(
                        f,
                        cases.iter().map(|(p, g, b)| {
                            format!(
                                "{} {} ->\n\t{}",
                                with_module(*p, m),
                                with_module(*g, m),
                                with_module(*b, m)
                            )
                        }),
                        ";\n\t",
                    )?;
                }
                if let Some(body) = after {
                    write!(f, "\nafter\n")?;
                    body.fmt(m, f)?;
                }
                write!(f, "\nend)")
            }
            Expr::Maybe(exprs, else_section) => {
                write!(f, "(maybe \n\t")?;
                write_list_ast_nodes(f, m, exprs, ",\n\t")?;
                if let Some(else_cases) = else_section {
                    write!(f, "\nelse\n\t")?;
                    write_list_strings(
                        f,
                        else_cases.iter().map(|(p, g, b)| {
                            format!(
                                "{} {} ->\n\t{}",
                                with_module(*p, m),
                                with_module(*g, m),
                                with_module(*b, m)
                            )
                        }),
                        ";\n\t",
                    )?;
                }
                write!(f, "\n\tend)")
            }
            Expr::Block(b) => {
                write!(f, "begin {} end", with_module(*b, m))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeSpecifier {
    // Same as Integer in practice, but prints nothing
    Default {
        signedness: Option<Signedness>,
        endianness: Option<Endianness>,
        unit: Option<u8>,
    },
    Integer {
        signedness: Option<Signedness>,
        endianness: Option<Endianness>,
        unit: Option<u8>,
    },
    Float {
        endianness: Option<Endianness>,
        unit: Option<u8>,
    },
    Binary,
    // alias of Binary
    Bytes,
    Bitstring,
    // alias of Bitstring
    Bits,
    Utf8,
    Utf16 {
        endianness: Option<Endianness>,
    },
    Utf32 {
        endianness: Option<Endianness>,
    },
}
// Only matters for matching
#[derive(Copy, Clone, Debug)]
pub enum Signedness {
    Signed,
    Unsigned,
}
#[derive(Copy, Clone, Debug)]
pub enum Endianness {
    Big,
    Little,
    Native,
}
impl SizedAst for TypeSpecifier {
    fn size(&self, _: &Module) -> ASTSize {
        match self {
            TypeSpecifier::Default {
                signedness,
                endianness,
                unit,
            } => {
                (if signedness.is_some() { 1 } else { 0 })
                    + if endianness.is_some() { 1 } else { 0 }
                    + if unit.is_some() { 1 } else { 0 }
            }
            TypeSpecifier::Integer {
                signedness,
                endianness,
                unit,
            } => {
                1 + if signedness.is_some() { 1 } else { 0 }
                    + if endianness.is_some() { 1 } else { 0 }
                    + if unit.is_some() { 1 } else { 0 }
            }
            TypeSpecifier::Float { endianness, unit } => {
                1 + if endianness.is_some() { 1 } else { 0 } + if unit.is_some() { 1 } else { 0 }
            }
            TypeSpecifier::Binary
            | TypeSpecifier::Bytes
            | TypeSpecifier::Bitstring
            | TypeSpecifier::Bits
            | TypeSpecifier::Utf8 => 1,
            TypeSpecifier::Utf16 { endianness } | TypeSpecifier::Utf32 { endianness } => {
                1 + if endianness.is_some() { 1 } else { 0 }
            }
        }
    }
}
impl fmt::Display for TypeSpecifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let TypeSpecifier::Default {
            signedness: None,
            endianness: None,
            unit: None,
        } = self
        {
            return Ok(());
        }
        write!(f, "/")?;
        let (base_type_string, signedness, endianness, unit) = match self {
            TypeSpecifier::Default {
                signedness,
                endianness,
                unit,
            } => (None, *signedness, *endianness, *unit),
            TypeSpecifier::Integer {
                signedness,
                endianness,
                unit,
            } => (Some("integer"), *signedness, *endianness, *unit),
            TypeSpecifier::Float { endianness, unit } => (Some("float"), None, *endianness, *unit),
            TypeSpecifier::Binary => (Some("binary"), None, None, None),
            TypeSpecifier::Bytes => (Some("bytes"), None, None, None),
            TypeSpecifier::Bitstring => (Some("bitstring"), None, None, None),
            TypeSpecifier::Bits => (Some("bits"), None, None, None),
            TypeSpecifier::Utf8 => (Some("utf8"), None, None, None),
            TypeSpecifier::Utf16 { endianness } => (Some("utf16"), None, *endianness, None),
            TypeSpecifier::Utf32 { endianness } => (Some("utf32"), None, *endianness, None),
        };
        let mut strings = Vec::new();
        if let Some(str) = base_type_string {
            strings.push(str.to_string());
        }
        if let Some(s) = signedness {
            strings.push(s.to_string());
        }
        if let Some(e) = endianness {
            strings.push(e.to_string());
        }
        if let Some(u) = unit {
            // "The allowed range is 1..256"
            strings.push(format!("unit:{}", (u as u32 + 1)));
        }
        write_list_strings(f, strings.into_iter(), "-")
    }
}
impl fmt::Display for Signedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signedness::Signed => write!(f, "signed"),
            Signedness::Unsigned => write!(f, "unsigned"),
        }
    }
}
impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endianness::Big => write!(f, "big"),
            Endianness::Little => write!(f, "little"),
            Endianness::Native => write!(f, "native"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ComprehensionKind {
    List,
    Bitstring,
}

#[derive(Clone, Debug, Copy)]
pub enum ComprehensionElement {
    ListGenerator(PatternId, ExprId),
    BitstringGenerator(PatternId, ExprId),
    MapGenerator(PatternId, PatternId, ExprId),
    Filter(ExprId),
}
impl SizedAst for ComprehensionElement {
    fn size(&self, m: &Module) -> ASTSize {
        match self {
            ComprehensionElement::ListGenerator(p, e) => 1 + p.size(m) + e.size(m),
            ComprehensionElement::BitstringGenerator(p, e) => 1 + p.size(m) + e.size(m),
            ComprehensionElement::MapGenerator(k, v, e) => 1 + k.size(m) + v.size(m) + e.size(m),
            ComprehensionElement::Filter(e) => e.size(m),
        }
    }
}
impl AstNode for ComprehensionElement {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComprehensionElement::ListGenerator(p, e) => {
                write!(f, "{} <- {}", with_module(*p, m), with_module(*e, m))
            }
            ComprehensionElement::BitstringGenerator(p, e) => {
                write!(f, "{} <= {}", with_module(*p, m), with_module(*e, m))
            }
            ComprehensionElement::MapGenerator(k, v, e) => {
                write!(
                    f,
                    "{} := {} <- {}",
                    with_module(*k, m),
                    with_module(*v, m),
                    with_module(*e, m)
                )
            }
            ComprehensionElement::Filter(e) => e.fmt(m, f),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MaybeExpr {
    Normal(ExprId),
    MaybeAssignment(PatternId, ExprId),
}
impl SizedAst for MaybeExpr {
    fn size(&self, m: &Module) -> ASTSize {
        match self {
            MaybeExpr::MaybeAssignment(p, e) => 1 + p.size(m) + e.size(m),
            MaybeExpr::Normal(e) => e.size(m),
        }
    }
}
impl AstNode for MaybeExpr {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeExpr::Normal(e) => e.fmt(m, f),
            MaybeExpr::MaybeAssignment(p, e) => {
                write!(f, "{} ?= {}", with_module(*p, m), with_module(*e, m))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub exprs: Vec<ExprId>,
}
impl SizedAst for Body {
    fn size(&self, module: &Module) -> ASTSize {
        self.exprs.size(module)
    }
}
impl AstNode for Body {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_list_ast_nodes(f, m, &self.exprs, ",\n\t")
    }
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Nil(),
    Atom(Atom),
    Integer(BigInt),
    Underscore(),
    NamedVar(VarNum),
    // p1 = p2
    EqualPatterns(PatternId, PatternId),
    Tuple(Vec<PatternId>),
    // [H | T]
    List(PatternId, PatternId),
    Bitstring(Vec<(PatternId, Option<ExprId>, TypeSpecifier)>),
    Map(Vec<(ExprId, PatternId)>),
}
impl SizedAst for Pattern {
    fn size(&self, module: &Module) -> ASTSize {
        match self {
            Pattern::EqualPatterns(p1, p2) => 1 + p1.size(module) + p2.size(module),
            Pattern::Tuple(patterns) => patterns.size(module),
            Pattern::List(head, tail) => 1 + head.size(module) + tail.size(module),
            Pattern::Bitstring(elements) => 1 + elements.size(module),
            Pattern::Map(mappings) => 1 + mappings.size(module),
            Pattern::Nil()
            | Pattern::Atom(_)
            | Pattern::Integer(_)
            | Pattern::Underscore()
            | Pattern::NamedVar(_) => 1,
        }
    }
}
impl AstNode for Pattern {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Underscore() => write!(f, "_"),
            Pattern::NamedVar(v) => write!(f, "_V{}", v), // TODO: add some kind of shortcut for _V{}?
            Pattern::EqualPatterns(p1, p2) => {
                write!(f, "({} = {})", with_module(*p1, m), with_module(*p2, m))
            }
            Pattern::Tuple(ps) => {
                write!(f, "{{")?;
                write_list_ast_nodes(f, m, ps, ", ")?;
                write!(f, "}}")
            }
            Pattern::List(h, t) => {
                write!(f, "[{} | {}]", with_module(*h, m), with_module(*t, m))
            }
            Pattern::Bitstring(elements) => {
                write!(f, "<<")?;
                write_list_strings(
                    f,
                    elements.iter().map(|(pattern, size, types)| {
                        let size = size
                            .map(|s| format!(":({})", with_module(s, m)))
                            .unwrap_or_default();
                        format!("{}{}{}", with_module(*pattern, m), size, types)
                    }),
                    ", ",
                )?;
                write!(f, ">>")
            }
            Pattern::Map(pairs) => {
                write!(f, "#{{")?;
                write_list_strings(
                    f,
                    pairs.iter().map(|(key, value)| {
                        format!("{} := {}", with_module(*key, m), with_module(*value, m))
                    }),
                    ", ",
                )?;
                write!(f, "}}")
            }
            Pattern::Nil() => write!(f, "[]"),
            Pattern::Atom(a) => write!(f, "{}", a),
            Pattern::Integer(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Default)]
pub struct GuardSeq {
    pub guards: Vec<ExprId>,
}
impl SizedAst for GuardSeq {
    fn size(&self, module: &Module) -> ASTSize {
        self.guards.size(module)
    }
}
impl AstNode for GuardSeq {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.guards.is_empty() {
            write!(f, " when ")?;
            write_list_ast_nodes(f, m, &self.guards, "; ")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct FunctionClause {
    pub name: String,
    pub args: Vec<PatternId>,
    pub guard: GuardSeqId,
    pub body: BodyId,
}
impl SizedAst for FunctionClause {
    fn size(&self, module: &Module) -> ASTSize {
        1 + self.args.size(module) + self.guard.size(module) + self.body.size(module)
    }
}
impl AstNode for FunctionClause {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        write_list_ast_nodes(f, m, &self.args, ", ")?;
        write!(
            f,
            ") {} -> \n\t{}",
            with_module(self.guard, m),
            with_module(self.body, m)
        )
    }
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    // Must be non-empty
    // Each clause must have the right arity and name
    // TODO: consider using richer types to represent this kind of constraints
    pub clauses: Vec<FunctionClauseId>,
    pub name: String,
    pub arity: Arity,
}
impl SizedAst for FunctionDeclaration {
    fn size(&self, module: &Module) -> ASTSize {
        self.clauses.size(module)
    }
}
impl AstNode for FunctionDeclaration {
    fn fmt(&self, m: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_list_ast_nodes(f, m, &self.clauses, ";\n")?;
        write!(f, ".")
    }
}

#[derive(Debug)]
pub struct Module<'a> {
    pub module_name: &'a str,
    pub initial_seed: u64,
    pub functions: Vec<FunctionDeclaration>,
    config: Config,
    patterns: Vec<Pattern>,
    exprs: Vec<Expr>,
    function_clauses: Vec<FunctionClause>,
    bodies: Vec<Body>,
    guard_seqs: Vec<GuardSeq>,
}
impl fmt::Display for Module<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = self.config;
        write!(f, "% initial seed: {}\n", self.initial_seed)?;
        write!(f, "% max-size: {}\n", config.max_size)?;
        write!(f, "% max-recursion-depth: {}\n", config.max_recursion_depth)?;
        write!(f, "% disable-shadowing: {}\n", config.disable_shadowing)?;
        write!(f, "% disable-maybe: {}\n", config.disable_maybe)?;
        write!(f, "% deterministic: {}\n", config.deterministic)?;
        write!(f, "% wrapper: {}\n", config.wrapper_mode)?;
        if !config.disable_maybe {
            write!(f, "-feature(maybe_expr, enable).\n")?;
        }
        write!(f, "-module({}).\n", self.module_name)?;
        write!(f, "-compile([export_all]).")?;
        for func_decl in &self.functions {
            write!(f, "\n\n")?;
            func_decl.fmt(self, f)?;
        }
        Ok(())
    }
}
impl<'a> Module<'a> {
    pub fn new(
        module_name: &'a str,
        initial_seed: u64,
        config: Config,
        functions: Vec<FunctionDeclaration>,
    ) -> Self {
        Self {
            module_name,
            initial_seed,
            config,
            functions,
            patterns: Vec::new(),
            exprs: Vec::new(),
            function_clauses: Vec::new(),
            bodies: Vec::new(),
            guard_seqs: Vec::new(),
        }
    }
    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }
    pub fn expr_mut(&mut self, id: ExprId) -> &mut Expr {
        &mut self.exprs[id.0 as usize]
    }
    pub fn add_expr(&mut self, e: Expr) -> ExprId {
        self.exprs.push(e);
        ExprId((self.exprs.len() - 1).try_into().unwrap())
    }
    pub fn pattern(&self, id: PatternId) -> &Pattern {
        &self.patterns[id.0 as usize]
    }
    pub fn pattern_mut(&mut self, id: PatternId) -> &mut Pattern {
        &mut self.patterns[id.0 as usize]
    }
    pub fn add_pattern(&mut self, p: Pattern) -> PatternId {
        self.patterns.push(p);
        PatternId((self.patterns.len() - 1).try_into().unwrap())
    }
    pub fn function_clause(&self, id: FunctionClauseId) -> &FunctionClause {
        &self.function_clauses[id.0 as usize]
    }
    pub fn function_clause_mut(&mut self, id: FunctionClauseId) -> &mut FunctionClause {
        &mut self.function_clauses[id.0 as usize]
    }
    pub fn add_function_clause(&mut self, p: FunctionClause) -> FunctionClauseId {
        self.function_clauses.push(p);
        FunctionClauseId((self.function_clauses.len() - 1).try_into().unwrap())
    }
    pub fn body(&self, id: BodyId) -> &Body {
        &self.bodies[id.0 as usize]
    }
    pub fn body_mut(&mut self, id: BodyId) -> &mut Body {
        &mut self.bodies[id.0 as usize]
    }
    pub fn add_body(&mut self, p: Body) -> BodyId {
        self.bodies.push(p);
        BodyId((self.bodies.len() - 1).try_into().unwrap())
    }
    pub fn guard_seq(&self, id: GuardSeqId) -> &GuardSeq {
        &self.guard_seqs[id.0 as usize]
    }
    pub fn guard_seq_mut(&mut self, id: GuardSeqId) -> &mut GuardSeq {
        &mut self.guard_seqs[id.0 as usize]
    }
    pub fn add_guard_seq(&mut self, p: GuardSeq) -> GuardSeqId {
        self.guard_seqs.push(p);
        GuardSeqId((self.guard_seqs.len() - 1).try_into().unwrap())
    }
}

pub trait NodeId {
    type Node: AstNode;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node;
}
impl<T: NodeId> SizedAst for T {
    fn size(&self, module: &Module) -> ASTSize {
        self.get(module).size(module)
    }
}
impl<T: NodeId> AstNode for T {
    fn fmt(&self, module: &Module, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get(module).fmt(module, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExprId(u32);
impl NodeId for ExprId {
    type Node = Expr;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node {
        module.expr(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PatternId(u32);
impl NodeId for PatternId {
    type Node = Pattern;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node {
        module.pattern(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionClauseId(u32);
impl NodeId for FunctionClauseId {
    type Node = FunctionClause;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node {
        module.function_clause(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BodyId(u32);
impl NodeId for BodyId {
    type Node = Body;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node {
        module.body(*self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GuardSeqId(u32);
impl NodeId for GuardSeqId {
    type Node = GuardSeq;
    fn get<'a>(&self, module: &'a Module) -> &'a Self::Node {
        module.guard_seq(*self)
    }
}
