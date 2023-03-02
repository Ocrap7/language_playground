use core::fmt;

use tl_util::format::{NodeDisplay, TreeDisplay};

use crate::token::{Range, SpannedToken, Token, Unit};

pub trait AstNode: TreeDisplay {
    fn get_range(&self) -> Range;
}

macro_rules! addup {
    ($($e:expr),*) => {{
        $((if let Some(_) = $e { 1 } else { 0 })+)* 0
    }};
}

macro_rules! switchon {
    ($index:expr, $($e:expr),*) => {{
        let mut ind = 0;
        $(if let Some(v) = $e {
            if $index == ind {
                return Some(v)
            }
            ind += 1;
        })*
        ind
    }};
}

impl AstNode for SpannedToken {
    fn get_range(&self) -> Range {
        self.0.into()
    }
}

#[derive(Clone)]
pub struct PunctuationList<T: AstNode> {
    tokens: Vec<(T, Option<SpannedToken>)>,
}

impl<T: AstNode> Default for PunctuationList<T> {
    fn default() -> Self {
        Self {
            tokens: Vec::default(),
        }
    }
}

impl<T: AstNode> PunctuationList<T> {
    pub fn push(&mut self, val: T, separator: Option<SpannedToken>) {
        self.tokens.push((val, separator))
    }

    pub fn push_sep(&mut self, val: T, separator: SpannedToken) {
        self.tokens.push((val, Some(separator)))
    }

    pub fn push_term(&mut self, val: T) {
        self.tokens.push((val, None))
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &T> + '_ {
        self.tokens.iter().map(|(v, _)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(T, Option<SpannedToken>)> + '_ {
        self.tokens.iter()
    }

    pub fn take(self) -> Vec<(T, Option<SpannedToken>)> {
        self.tokens
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

impl<T> AstNode for PunctuationList<T>
where
    T: AstNode,
{
    fn get_range(&self) -> Range {
        match (self.iter().next(), self.iter().last()) {
            (Some((_, Some(f))), Some((_, Some(l)))) => Range::from((*f.span(), *l.span())),
            (Some((_, Some(f))), Some((l, _))) => Range::from((*f.span(), &l.get_range())),
            (Some((f, _)), Some((_, Some(l)))) => Range::from((&f.get_range(), *l.span())),
            (Some((f, _)), Some((l, _))) => Range::from((&f.get_range(), &l.get_range())),
            _ => Range::default(),
        }
    }
}

impl<T> NodeDisplay for PunctuationList<T>
where
    T: NodeDisplay + AstNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Punctuation List")?;
        write!(f, " {}", self.tokens.len())
    }
}

impl<T> TreeDisplay for PunctuationList<T>
where
    T: TreeDisplay + AstNode,
{
    fn num_children(&self) -> usize {
        if let Some((_, Some(_))) = self.tokens.last() {
            self.tokens.len() * 2
        } else if !self.tokens.is_empty() {
            self.tokens.len() * 2 - 1
        } else {
            0
        }
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        let p = &self.tokens[index / 2];
        if index % 2 == 0 {
            Some(&p.0)
        } else {
            Some(p.1.as_ref().unwrap())
        }
    }
}

impl<T: PartialEq + AstNode> PartialEq for PunctuationList<T> {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.iter_items().zip(other.iter_items()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct ParamaterList {
    pub range: Range,
    pub items: PunctuationList<Param>,
}

impl AstNode for ParamaterList {
    fn get_range(&self) -> Range {
        self.range
    }
}

impl ParamaterList {
    pub fn iter_items(&self) -> impl Iterator<Item = &Param> + '_ {
        self.items.iter_items()
    }
}

impl NodeDisplay for ParamaterList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Element Parameters")
    }
}

impl TreeDisplay for ParamaterList {
    fn num_children(&self) -> usize {
        2
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match index {
            0 => Some(&self.range),
            1 => Some(&self.items),
            _ => panic!(),
        }
    }
}

impl PartialEq for ParamaterList {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.iter_items().zip(other.iter_items()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct Param {
    pub ty: Option<Type>,
    pub name: Option<SpannedToken>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl AstNode for Param {
    fn get_range(&self) -> Range {
        match (&self.name, &self.ty) {
            (Some(name), None) => Range::from(*name.span()),
            (Some(name), Some(value)) => Range::from((name, &value.get_range())),
            _ => Range::default(),
        }
    }
}

impl Param {
    pub fn name(&self) -> &String {
        match &self.name {
            Some(SpannedToken(_, Token::Ident(s))) => s,
            _ => panic!(),
        }
    }
}

impl NodeDisplay for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Parameter")
    }
}

impl TreeDisplay for Param {
    fn num_children(&self) -> usize {
        addup!(self.ty, self.name)
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        switchon!(index, &self.ty, &self.name);
        None
    }
}

#[derive(Clone)]
pub struct ArgList {
    pub range: Range,
    pub items: PunctuationList<Expression>,
}

impl AstNode for ArgList {
    fn get_range(&self) -> Range {
        self.range
    }
}

impl ArgList {
    pub fn iter_items(&self) -> impl Iterator<Item = &Expression> + '_ {
        self.items.iter_items()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl NodeDisplay for ArgList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Arg Parameters")
    }
}

impl TreeDisplay for ArgList {
    fn num_children(&self) -> usize {
        2
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match index {
            0 => Some(&self.range),
            1 => Some(&self.items),
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct EnclosedList<T: AstNode> {
    pub open: SpannedToken,
    pub items: PunctuationList<T>,
    pub close: SpannedToken,
}

impl<T: AstNode> AstNode for EnclosedList<T> {
    fn get_range(&self) -> Range {
        Range::from((&self.open, &self.close))
    }
}

impl<T: AstNode> EnclosedList<T> {
    pub fn iter_items(&self) -> impl Iterator<Item = &T> + '_ {
        self.items.iter_items()
    }
}

impl<T: AstNode> NodeDisplay for EnclosedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Enclosed List")
    }
}

impl<T: AstNode> TreeDisplay for EnclosedList<T> {
    fn num_children(&self) -> usize {
        1
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match index {
            0 => Some(&self.items),
            _ => None,
        }
    }
}

impl<T: PartialEq + AstNode> PartialEq for EnclosedList<T> {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.iter_items().zip(other.iter_items()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub enum Type {
    Integer {
        width: u8,
        signed: bool,
        token: SpannedToken,
    },
    Float {
        width: u8,
        token: SpannedToken,
    },
    Boolean(SpannedToken),
    Char(SpannedToken),
    Ident(SpannedToken),
    Ref {
        ref_token: SpannedToken,
        base_type: Option<Box<Type>>,
    },
    Array(EnclosedList<Type>),
    Union(PunctuationList<Type>),
    Tuple(EnclosedList<Type>),
    Generic {
        base_type: Option<Box<Type>>,
        list: EnclosedList<Type>,
    },
    Expression(Box<Expression>),
    Function {
        parameters: ParamaterList,
        return_type: Option<(SpannedToken, Box<Type>)>,
    },
    Option {
        base_type: Option<Box<Type>>,
        question: SpannedToken,
    },
    Result {
        error: SpannedToken,
        base_type: Option<Box<Type>>,
    },
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Integer {
                    width: l_width,
                    signed: l_signed,
                    token: l_token,
                },
                Self::Integer {
                    width: r_width,
                    signed: r_signed,
                    token: r_token,
                },
            ) => l_width == r_width && l_signed == r_signed,
            (
                Self::Float {
                    width: l_width,
                    token: l_token,
                },
                Self::Float {
                    width: r_width,
                    token: r_token,
                },
            ) => l_width == r_width,
            (Self::Boolean(l0), Self::Boolean(r0)) => true,
            (Self::Char(l0), Self::Char(r0)) => true,
            (Self::Ident(l0), Self::Ident(r0)) => l0.as_str() == r0.as_str(),
            (
                Self::Ref {
                    ref_token: l_ref_token,
                    base_type: Some(l_base_type),
                },
                Self::Ref {
                    ref_token: r_ref_token,
                    base_type: Some(r_base_type),
                },
            ) => l_base_type == r_base_type,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Union(l0), Self::Union(r0)) => l0 == r0,
            (Self::Tuple(l0), Self::Tuple(r0)) => l0 == r0,
            (
                Self::Generic {
                    base_type: Some(btl0),
                    list: l0,
                },
                Self::Generic {
                    base_type: Some(btr0),
                    list: r0,
                },
            ) => l0 == r0 && btl0 == btr0,
            // (Self::Expression(l0), Self::Expression(r0)) => l0 == r0,
            (
                Self::Function {
                    parameters: l_parameters,
                    return_type: Some(l_return_type),
                },
                Self::Function {
                    parameters: r_parameters,
                    return_type: Some(r_return_type),
                },
            ) => l_parameters == r_parameters && l_return_type.1 == r_return_type.1,
            (
                Self::Function {
                    parameters: l_parameters,
                    return_type: None,
                },
                Self::Function {
                    parameters: r_parameters,
                    return_type: None,
                },
            ) => l_parameters == r_parameters,
            (
                Self::Option {
                    base_type: Some(l_ty),
                    question: l_question,
                },
                Self::Option {
                    base_type: Some(r_ty),
                    question: r_question,
                },
            ) => l_ty == r_ty,
            (
                Self::Result {
                    error: l_error,
                    base_type: Some(l_ty),
                },
                Self::Result {
                    error: r_error,
                    base_type: Some(r_ty),
                },
            ) => l_ty == r_ty,
            _ => false,
        }
    }
}

// impl PartialEq for Type {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (
//                 Self::Integer {
//                     width: l_width,
//                     signed: l_signed,
//                     ..
//                 },
//                 Self::Integer {
//                     width: r_width,
//                     signed: r_signed,
//                     ..
//                 },
//             ) => l_width == r_width && l_signed == r_signed,
//             (Self::Float { width: l_width, .. }, Self::Float { width: r_width, .. }) => {
//                 l_width == r_width
//             }
//             (
//                 Self::Ident(SpannedToken(_, Token::Ident(a))),
//                 Self::Ident(SpannedToken(_, Token::Ident(b))),
//             ) => a == b,
//             _ => false,
//         }
//     }
// }

impl AstNode for Type {
    fn get_range(&self) -> Range {
        match self {
            Self::Integer { token, .. } => token.span().into(),
            Self::Float { token, .. } => token.span().into(),
            Self::Boolean(tok) => tok.span().into(),
            Self::Char(tok) => tok.span().into(),
            Self::Ident(ident) => ident.span().into(),
            Self::Array(a) => a.get_range(),
            Self::Union(a) => a.get_range(),
            Self::Tuple(a) => a.get_range(),
            Self::Generic {
                base_type: Some(base_type),
                list,
            } => Range::from((&base_type.get_range(), &list.get_range())),
            Self::Generic { list, .. } => list.get_range(),
            Self::Expression(a) => a.get_range(),
            Self::Function {
                parameters,
                return_type: None,
            } => parameters.get_range(),
            Self::Function {
                parameters,
                return_type: Some((_, ty)),
            } => Range::from((&parameters.get_range(), &ty.get_range())),
            Self::Option {
                base_type: Some(ty),
                question,
            } => Range::from((&ty.get_range(), &question.get_range())),
            Self::Option { question, .. } => question.get_range(),
            Self::Result {
                base_type: Some(ty),
                error,
            } => Range::from((&error.get_range(), &ty.get_range())),
            Self::Result { error, .. } => error.get_range(),
            Self::Ref {
                ref_token,
                base_type: Some(base_type),
            } => Range::from((&base_type.get_range(), &ref_token.get_range())),
            Self::Ref { ref_token, .. } => ref_token.get_range(),
        }
    }
}

impl NodeDisplay for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Float { width, .. } => write!(f, "f{width}"),
            Self::Integer {
                width,
                signed: true,
                ..
            } => write!(f, "i{width}"),
            Self::Integer {
                width,
                signed: false,
                ..
            } => write!(f, "u{width}"),
            Self::Boolean(_) => f.write_str("bool"),
            Self::Char(_) => f.write_str("char"),
            Self::Ident(ident) => f.write_str(ident.as_str()),
            Self::Array(a) => {
                f.write_str("Array")
                // write!(f, "{}", a.open.as_op_str())?;
                // let p: String = a
                //     .items
                //     .iter_items()
                //     .map(|f| format!("{}", f.format()))
                //     .intersperse(", ".to_string())
                //     .collect();
                // write!(f, "{}{}", p, a.close.as_op_str())
            }
            Self::Union(u) => {
                f.write_str("Union")
                // for item in u
                //     .iter_items()
                //     .map(|f| format!("{}", f.format()))
                //     .intersperse(" | ".to_string())
                // {
                //     f.write_str(item.as_str())?;
                // }
                // Ok(())
            }
            Self::Tuple(a) => {
                f.write_str("Tuple")
                // write!(f, "{}", a.open.as_op_str())?;
                // let p: String = a
                //     .items
                //     .iter_items()
                //     .map(|f| format!("{}", f.format()))
                //     .intersperse(", ".to_string())
                //     .collect();
                // write!(f, "{}{}", p, a.close.as_op_str())
            }
            Self::Generic { .. } => {
                f.write_str("Generic")
                // write!(f, "{}", a.open.as_op_str())?;
                // let p: String = a
                //     .items
                //     .iter_items()
                //     .map(|f| format!("{}", f.format()))
                //     .intersperse(", ".to_string())
                //     .collect();
                // write!(f, "{}{}", p, a.close.as_op_str())
            }
            Self::Expression(e) => f.write_str("Expression"),
            Self::Function {
                parameters,
                ..
                // return_type: None,
            } => {
                f.write_str("Function")
                // f.write_str("(")?;

                // let err: fmt::Result = parameters
                //     .iter_items()
                //     .map(|f| format!("{} {}", f.ty.as_ref().unwrap().format(), f.name()))
                //     .intersperse(", ".to_string())
                //     .map(|st| f.write_str(st.as_str()))
                //     .collect();
                // err?;

                // f.write_str(")")
            }
            // Self::Function {
            //     parameters,
            //     return_type: Some((_, ret)),
            // } => {
            //     f.write_str("(")?;

            //     let err: fmt::Result = parameters
            //         .iter_items()
            //         .map(|f| format!("{} {}", f.ty.as_ref().unwrap().format(), f.name()))
            //         .intersperse(", ".to_string())
            //         .map(|st| f.write_str(st.as_str()))
            //         .collect();
            //     err?;

            //     f.write_str(") =>");

            //     ret.fmt(f)
            // }
            Self::Option { base_type: ty, question } => {
                f.write_str("Optional")
                // if let Some(ty) = &ty {
                //     ty.fmt(f)?;
                // }
                // f.write_str(question.as_op_str())
            }
            Self::Result { base_type: ty, error } => {
                f.write_str("Result")
                // if let Some(ty) = &ty {
                //     ty.fmt(f)?;
                // }
                // f.write_str(error.as_op_str())
            }
            Self::Ref {
                ref_token,
                base_type,
            } => {
                f.write_str("Reference")

                // if let Some(base_type) = &base_type {
                //     base_type.fmt(f)?;
                // }
                // f.write_str(ref_token.as_op_str())
            }
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Type as NodeDisplay>::fmt(self, f)
    }
}

impl TreeDisplay for Type {
    fn num_children(&self) -> usize {
        match self {
            Type::Array(a) => a.items.num_children(),
            Type::Union(a) => a.num_children(),
            Type::Tuple(a) => a.items.num_children(),
            Type::Generic {
                base_type: Some(_), ..
            } => 2,
            Type::Generic { .. } => 1,
            Type::Expression(e) => 1,
            Type::Option { .. } => 1,
            Type::Result { .. } => 1,
            Type::Ref { .. } => 1,
            _ => 0,
        }
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match self {
            Type::Array(a) => a.items.child_at(index),
            Type::Union(a) => a.child_at(index),
            Type::Tuple(a) => a.items.child_at(index),
            Type::Generic {
                base_type: Some(base_type),
                list,
            } => match index {
                0 => Some(&**base_type),
                1 => Some(list),
                _ => None,
            },
            Type::Generic { list, .. } => Some(list),
            Type::Expression(e) => Some(&**e),
            Type::Option { base_type: ty, .. } => ty.as_ref().map::<&dyn TreeDisplay, _>(|f| &**f),
            Type::Result { base_type: ty, .. } => ty.as_ref().map::<&dyn TreeDisplay, _>(|f| &**f),
            Type::Ref { base_type, .. } => base_type.as_ref().map::<&dyn TreeDisplay, _>(|f| &**f),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum GenericParameter {
    Unbounded(SpannedToken),
    Bounded {
        ident: SpannedToken,
        colon: SpannedToken,
        bounds: PunctuationList<SpannedToken>,
    },
}

impl AstNode for GenericParameter {
    fn get_range(&self) -> Range {
        match self {
            GenericParameter::Unbounded(u) => u.get_range(),
            GenericParameter::Bounded { ident, bounds, .. } => {
                Range::from((*ident.span(), &bounds.get_range()))
            }
        }
    }
}

impl NodeDisplay for GenericParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenericParameter::Unbounded(u) => f.write_str(u.as_str()),
            GenericParameter::Bounded { ident, .. } => f.write_str(ident.as_str()),
        }
    }
}

impl TreeDisplay for GenericParameter {
    fn num_children(&self) -> usize {
        match self {
            GenericParameter::Bounded { bounds, .. } => bounds.num_children(),
            _ => 0,
        }
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay<()>> {
        match self {
            GenericParameter::Bounded { bounds, .. } => bounds.child_at(index),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum ParsedTemplate {
    String(SpannedToken),
    Template(Box<Expression>, SpannedToken, SpannedToken),
}

impl NodeDisplay for ParsedTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParsedTemplate::String(l) => write!(f, "Literal: `{}`", l.as_str()),
            ParsedTemplate::Template(t, _, _) => write!(f, "Value"),
        }
    }
}

impl TreeDisplay for ParsedTemplate {
    fn num_children(&self) -> usize {
        match self {
            ParsedTemplate::Template(_, _, _) => 1,
            _ => 0,
        }
    }

    fn child_at(&self, _index: usize) -> Option<&dyn TreeDisplay<()>> {
        match self {
            ParsedTemplate::Template(e, _, _) => Some(&**e),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ParsedTemplateString(pub Vec<ParsedTemplate>);

impl NodeDisplay for ParsedTemplateString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Parsed Template String")
    }
}

impl TreeDisplay for ParsedTemplateString {
    fn num_children(&self) -> usize {
        self.0.len()
    }

    fn child_at(&self, _index: usize) -> Option<&dyn TreeDisplay<()>> {
        self.0.get(_index).map::<&dyn TreeDisplay<()>, _>(|t| t)
    }
}

#[derive(Clone)]
pub enum Expression {
    BinaryExpression {
        left: Option<Box<Expression>>,
        right: Option<Box<Expression>>,
        op_token: Option<SpannedToken>,
    },
    Integer(u64, Option<Unit>, SpannedToken),
    Float(f64, Option<Unit>, SpannedToken),
    Ident(SpannedToken),
    String(ParsedTemplateString, SpannedToken),
    FunctionCall {
        expr: Box<Expression>,
        args: ArgList,
    },
    Tuple(Vec<Expression>),
    Array {
        values: PunctuationList<Expression>,
        range: Range,
    },
    Function {
        parameters: ParamaterList,
        arrow: SpannedToken,
        return_parameters: ParamaterList,
        comma: Option<SpannedToken>,
        body: Option<Box<Statement>>,
    },
    Record {
        parameters: ParamaterList,
    },
}

impl Expression {
    pub fn as_function(&self) -> Option<(&Expression, &ArgList)> {
        match self {
            Expression::FunctionCall { expr, args } => Some((expr, args)),
            _ => None,
        }
    }
}

impl AstNode for Expression {
    fn get_range(&self) -> Range {
        match self {
            Self::Record { parameters } => parameters.get_range(),
            Self::Function {
                parameters,
                body: Some(body),
                ..
            } => Range::from((&parameters.get_range(), &body.get_range())),
            Self::Function {
                parameters,
                comma: Some(comma),
                ..
            } => Range::from((&parameters.get_range(), *comma.span())),
            Self::Function {
                parameters,
                return_parameters,
                ..
            } => Range::from((&parameters.get_range(), &return_parameters.get_range())),
            Self::BinaryExpression {
                left: Some(left),
                right: Some(right),
                ..
            } => Range::from((&left.get_range(), &right.get_range())),
            Self::BinaryExpression {
                left: Some(left),
                op_token: Some(op),
                ..
            } => Range::from((&left.get_range(), *op.span())),
            Self::BinaryExpression {
                op_token: Some(op),
                right: Some(right),
                ..
            } => Range::from((*op.span(), &right.get_range())),
            Self::BinaryExpression {
                op_token: Some(op), ..
            } => Range::from(*op.span()),
            Self::Tuple(s) => match (s.first(), s.last()) {
                (Some(s), Some(e)) => Range::from((&s.get_range(), &e.get_range())),
                _ => Range::default(),
            },
            Self::Array { range, .. } => *range,
            Self::Integer(_, _, s) => s.0.into(),
            Self::Float(_, _, s) => s.0.into(),
            Self::Ident(s) => s.0.into(),
            Self::FunctionCall { expr, args } => {
                Range::from((&expr.get_range(), &args.get_range()))
            }
            _ => Range::default(),
        }
    }
}

impl NodeDisplay for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Function { .. } => f.write_str("Function"),
            Self::Record { .. } => f.write_str("Record"),
            Self::BinaryExpression {
                op_token: Some(SpannedToken(_, Token::Operator(op))),
                ..
            } => write!(f, "BinExp {}", op.as_str()),
            Self::BinaryExpression { .. } => write!(f, "BinExp"),
            Self::Integer(i, Some(u), _) => write!(f, "{i}{u}"),
            Self::Float(i, Some(u), _) => write!(f, "{i}{u}"),
            Self::Integer(i, None, _) => write!(f, "{i}"),
            Self::Float(i, None, _) => write!(f, "{i}"),
            Self::Ident(SpannedToken(_, Token::Ident(i))) => write!(f, "{i}"),
            Self::String(_, _) => f.write_str("TemplateString"),
            Self::FunctionCall { .. } => write!(f, "FunctionCall"),
            Self::Array { .. } => f.write_str("Array"),
            _ => panic!(),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Expression as NodeDisplay>::fmt(self, f)
    }
}

impl TreeDisplay for Expression {
    fn num_children(&self) -> usize {
        match self {
            Self::Record { .. } => 1,
            Self::Function { body: Some(_), .. } => 3,
            Self::Function { .. } => 2,
            Self::FunctionCall { .. } => 2,
            Self::Array { values, .. } => values.num_children(),
            Self::BinaryExpression {
                left: Some(_),
                right: Some(_),
                ..
            } => 2,
            Self::BinaryExpression { left: Some(_), .. } => 1,
            Self::BinaryExpression { right: Some(_), .. } => 1,
            Self::String(p, _) => p.num_children(),
            _ => 0,
        }
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match self {
            Self::Record { parameters } => Some(parameters),
            Self::Function {
                parameters,
                return_parameters,
                body: Some(body),
                ..
            } => match index {
                0 => Some(parameters),
                1 => Some(return_parameters),
                2 => Some(&**body),
                _ => None,
            },
            Self::Function {
                parameters,
                return_parameters,
                ..
            } => match index {
                0 => Some(parameters),
                1 => Some(return_parameters),
                _ => None,
            },

            Self::FunctionCall { expr, args, .. } => match index {
                0 => Some(&**expr),
                1 => Some(args),
                _ => None,
            },
            Self::Array { values, .. } => values.child_at(index),
            Self::BinaryExpression {
                left: Some(l),
                right: Some(r),
                ..
            } => match index {
                0 => Some(&**l),
                1 => Some(&**r),
                _ => None,
            },
            Self::BinaryExpression { left: Some(l), .. } => Some(&**l),
            Self::BinaryExpression { right: Some(r), .. } => Some(&**r),
            Self::String(t, _) => t.child_at(index),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum Statement {
    Expression(Expression),
    Decleration {
        ident: SpannedToken,
        colon: SpannedToken,
        expr: Option<Expression>,
    },
    UseStatement {
        token: Option<SpannedToken>,
        args: PunctuationList<SpannedToken>,
    },
    List(PunctuationList<Statement>),
    TypeAlias {
        ty_tok: SpannedToken,
        ident: SpannedToken,
        generic: Option<EnclosedList<GenericParameter>>,
        eq: SpannedToken,
        ty: Box<Type>,
    },
}

impl AstNode for Statement {
    fn get_range(&self) -> Range {
        match self {
            Self::Expression(e) => e.get_range(),
            Self::UseStatement {
                token: Some(token),
                args,
            } => match args.iter().last() {
                Some((_, Some(tok))) => Range::from((*token.span(), *tok.span())),
                Some((tok, _)) => Range::from((*token.span(), *tok.span())),
                _ => Range::from(*token.span()),
            },
            Self::List(list) => list.get_range(),
            _ => Range::default(),
        }
    }
}

impl NodeDisplay for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Statement")
    }
}

impl TreeDisplay for Statement {
    fn num_children(&self) -> usize {
        match self {
            Self::Decleration { .. } => 2,
            Self::UseStatement { token, args } => addup!(token) + args.num_children(),
            Self::Expression(_) => 1,
            Self::List(list) => list.num_children(),
            Self::TypeAlias { generic: Some(_), .. } => 3,
            Self::TypeAlias { .. } => 2,
        }
    }

    fn child_at(&self, index: usize) -> Option<&dyn TreeDisplay> {
        match self {
            Self::Decleration {
                ident,
                expr: Some(expr),
                ..
            } => match index {
                0 => Some(ident),
                1 => Some(expr),
                _ => None,
            },
            Self::Decleration { ident, .. } => Some(ident),
            Self::UseStatement { token, args } => {
                let ind = switchon!(index, token);
                args.child_at(index - ind)
            }
            Self::Expression(e) => Some(e),
            Self::List(list) => list.child_at(index),

            Self::TypeAlias { ident, generic: Some(_), .. } if index == 0 => Some(ident),
            Self::TypeAlias { generic: Some(gen), .. } if index == 1 => Some(gen),
            Self::TypeAlias { generic: Some(_), ty, .. } if index == 2 => Some(&**ty),

            Self::TypeAlias { ident, .. } if index == 0 => Some(ident),
            Self::TypeAlias { ty, .. } if index == 1 => Some(&**ty),
            _ => None,
        }
    }

    fn child_at_bx<'b>(&'b self, index: usize) -> Box<dyn TreeDisplay + 'b> {
        match self {
            Self::Expression(e) => e.child_at_bx(index),
            _ => panic!(),
        }
    }
}
