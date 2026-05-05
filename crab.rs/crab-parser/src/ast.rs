#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelItem {
    Import(ImportStmt),
    Export(ExportStmt),
    FunctionDecl(FunctionDecl),
    ClassDecl(ClassDecl),
    MixinDecl(MixinDecl),
    ExtensionDecl(ExtensionDecl),
    TypeAlias(TypeAlias),
    Const(ConstDecl),
    CBlock(CBlockDecl),
    VarDecl(TopLevelVarDecl),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelVarDecl {
    pub name: String,
    pub typ: Type,
    pub value: Expr,
    pub is_final: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExportStmt {
    pub path: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CBlockDecl {
    pub code: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub path: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: FunctionBody,
    pub is_async: bool,
    pub is_generator: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Expression(Box<Expr>),
    Block(Vec<Statement>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
    pub is_required: bool,
    pub default_value: Option<Box<Expr>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<Box<Type>>,
    pub mixins: Vec<Type>,
    pub implements: Vec<Type>,
    pub fields: Vec<Field>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<MethodDecl>,
    pub is_sealed: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_interface: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub typ: Type,
    pub is_final: bool,
    pub is_static: bool,
    pub is_private: bool,
    pub initializer: Option<Box<Expr>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
    pub name: Option<String>,
    pub params: Vec<Parameter>,
    pub initializers: Vec<Initializer>,
    pub body: Vec<Statement>,
    pub has_super_delegation: bool,
    pub super_args: Vec<Expr>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Initializer {
    pub field: String,
    pub value: Expr,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: FunctionBody,
    pub is_static: bool,
    pub is_getter: bool,
    pub is_setter: bool,
    pub is_override: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixinDecl {
    pub name: String,
    pub on_type: Option<Type>,
    pub methods: Vec<MethodDecl>,
    pub fields: Vec<Field>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionDecl {
    pub name: Option<String>,
    pub on_type: Type,
    pub methods: Vec<MethodDecl>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub typ: Type,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: String,
    pub typ: Option<Type>,
    pub value: Expr,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Double,
    Bool,
    String,
    Void,
    Dynamic,
    Never,
    Custom(String),
    Nullable(Box<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Generic(String, Vec<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(Option<String>, Type)>),
    Future(Box<Type>),
    Stream(Box<Type>),
    Result(Box<Type>, Box<Type>),
    OptionT(Box<Type>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(VarDecl),
    FinalDecl(VarDecl),
    ConstDecl(ConstDecl),
    Expression(Expr),
    If(IfStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    Switch(SwitchStmt),
    Return(Option<Box<Expr>>),
    Break(Option<String>),
    Continue(Option<String>),
    Block(Vec<Statement>),
    Try(TryStmt),
    ThrowStmt(Expr),
}
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub name: String,
    pub typ: Option<Type>,
    pub value: Option<Box<Expr>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Box<Expr>,
    pub then_block: Vec<Statement>,
    pub else_if_blocks: Vec<(Box<Expr>, Vec<Statement>)>,
    pub else_block: Option<Vec<Statement>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub label: Option<String>,
    pub condition: Box<Expr>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStmt {
    pub label: Option<String>,
    pub body: Vec<Statement>,
    pub condition: Box<Expr>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub label: Option<String>,
    pub init_var: Option<(String, Option<Type>)>,
    pub init_expr: Option<Box<Expr>>,
    pub condition: Option<Box<Expr>>,
    pub update: Option<Box<Expr>>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ForInStmt {
    pub label: Option<String>,
    pub variable: String,
    pub iterable: Box<Expr>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub expr: Box<Expr>,
    pub cases: Vec<SwitchCase>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub pattern: SwitchPattern,
    pub guard: Option<Box<Expr>>,
    pub result: Expr,
}
#[derive(Debug, Clone, PartialEq)]
pub enum SwitchPattern {
    Literal(Expr),
    Default,
    Or(Vec<Box<SwitchPattern>>),
    Destructure(String, Vec<String>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub body: Vec<Statement>,
    pub catch_blocks: Vec<CatchBlock>,
    pub finally_block: Option<Vec<Statement>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CatchBlock {
    pub exception_type: Option<Type>,
    pub exception_var: Option<String>,
    pub body: Vec<Statement>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
    NullAware {
        object: Box<Expr>,
        property: String,
        args: Option<Vec<Expr>>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Cast {
        expr: Box<Expr>,
        typ: Type,
    },
    Is {
        expr: Box<Expr>,
        typ: Type,
        negated: bool,
    },
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expr>,
    },
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    SetLiteral(Vec<Expr>),
    Spread(Box<Expr>),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    CompoundAssign {
        target: Box<Expr>,
        op: BinaryOp,
        value: Box<Expr>,
    },
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    NullCoalesceAssign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    NullAssertion(Box<Expr>),
    StringInterpolation(Vec<StringPart>),
    New {
        class_name: String,
        constructor: Option<String>,
        args: Vec<Expr>,
    },
    This,
    Super {
        field_or_method: Option<String>,
    },
    Await(Box<Expr>),
    Propagate(Box<Expr>),
    ResultConstructor {
        variant: String,
        value: Box<Expr>,
    },
    OptionConstructor {
        variant: String,
        value: Option<Box<Expr>>,
    },
    Block(Vec<Statement>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Static(String),
    Interpolation(Box<Expr>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Double(f64),
    Bool(bool),
    String(String),
    Null,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Pow,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}
