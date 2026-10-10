use datafusion::arrow::datatypes::DataType;
use datafusion::logical_expr::Expr;
use rand::{Rng, rngs::StdRng};
use std::sync::{Arc, LazyLock};
use strum::{EnumIter, IntoEnumIterator};

use super::expr_impl::{
    AddExpr,
    AndExpr,
    CurrentDateExpr,
    CurrentTimeExpr,
    CurrentTimestampExpr,
    DateFormatExpr,
    DivExpr,
    // Comparison operators
    EqExpr,
    GtEqExpr,
    GtExpr,
    ILikeExpr,
    IsDistinctFromExpr,
    IsNotDistinctFromExpr,
    IsNotNullExpr,
    IsNullExpr,
    LikeExpr,
    LtEqExpr,
    LtExpr,
    ModExpr,
    MulExpr,
    NotEqExpr,
    NotExpr,
    NotILikeExpr,
    NotLikeExpr,
    NowExpr,
    OrExpr,
    RegexIMatchExpr,
    RegexMatchExpr,
    RegexNotIMatchExpr,
    RegexNotMatchExpr,
    SubExpr,
    ToCharExpr,
    ToDateExpr,
    ToLocalTimeExpr,
    ToTimestampExpr,
    ToTimestampMicrosExpr,
    ToTimestampMillisExpr,
    ToTimestampNanosExpr,
    ToTimestampSecondsExpr,
    ToUnixtimeExpr,
    TodayExpr,
};

/// A wrapper of datafusion expression
pub struct ExprWrapper {
    // Raw datafusion expression
    pub(crate) expr: BaseExpr,
    // All possible return types
    pub(crate) return_type: Vec<DataType>,
    // Either:
    // (SAME_AS_OUTPUT, INT/DOUBLE, INT/DOUBLE)
    // (SAME_AS_OUTPUT)
    pub(crate) inferred_child_signature: Vec<Vec<TypeGroup>>,
}

impl ExprWrapper {
    /// `ExprWrapper` itself defines what's a possible child signature for the expr,
    /// this function picks one valid signature from all possibilities.
    ///
    /// # Example
    ///
    /// For an expression wrapper like:
    /// ```text
    /// ExprWrapper {
    ///     expr: Add,
    ///     return_type: [Int64, Float64],
    ///     inferred_child_signature: [[SameAsOutput, SameAsOutput]]
    /// }
    /// ```
    ///
    /// When we want to generate a Float64 `Add` expression, this function will infer
    /// the child signature to be:
    ///
    /// [SameAsOutput, SameAsOutput] -> [Float64, Float64]
    // TODO: inject more randomness here (by generating a invalid signature)
    pub fn pick_child_signature(&self, output_type: DataType, rng: &mut StdRng) -> Vec<DataType> {
        // Pick one signature in TypeGroup like (SAME_AS_OUTPUT, INT/Float64, INT)
        let signature = &self.inferred_child_signature
            [rng.random_range(0..self.inferred_child_signature.len())];

        // Pick one valid type from each TypeGroup in the signature
        let picked_types = signature
            .iter()
            .map(|group| match group {
                TypeGroup::SameAsOutput => output_type.clone(),
                TypeGroup::Fixed(dt) => dt.clone(),
                TypeGroup::OneOf(dts) => dts[rng.random_range(0..dts.len())].clone(),
            })
            .collect();

        picked_types
    }
}

#[derive(Debug, Clone)]
pub enum TypeGroup {
    /// The type should be the same as the output type of the expression
    SameAsOutput,
    /// A fixed data type that must be used
    Fixed(DataType),
    /// A set of possible data types to choose from
    OneOf(Vec<DataType>),
}

/// BaseExpr can map to a datafusion expression, it's used to build a corresponding
/// datafusion expression.
#[derive(Debug, Clone, EnumIter)]
pub enum BaseExpr {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Not,
    IsNull,
    IsNotNull,
    // Comparison operators
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    IsDistinctFrom,
    IsNotDistinctFrom,
    Like,
    ILike,
    NotLike,
    NotILike,
    RegexMatch,
    RegexIMatch,
    RegexNotMatch,
    RegexNotIMatch,
    CurrentDate,
    CurrentTime,
    Now,
    CurrentTimestamp,
    ToChar,
    DateFormat,
    ToDate,
    ToLocalTime,
    ToTimestamp,
    ToTimestampMicros,
    ToTimestampMillis,
    ToTimestampNanos,
    ToTimestampSeconds,
    ToUnixtime,
    Today,
}

impl BaseExpr {
    pub fn to_impl(&self) -> Box<dyn BaseExprWithInfo> {
        match self {
            BaseExpr::Add => Box::new(AddExpr),
            BaseExpr::Sub => Box::new(SubExpr),
            BaseExpr::Mul => Box::new(MulExpr),
            BaseExpr::Div => Box::new(DivExpr),
            BaseExpr::Mod => Box::new(ModExpr),
            BaseExpr::And => Box::new(AndExpr),
            BaseExpr::Or => Box::new(OrExpr),
            BaseExpr::Not => Box::new(NotExpr),
            BaseExpr::IsNull => Box::new(IsNullExpr),
            BaseExpr::IsNotNull => Box::new(IsNotNullExpr),
            // Comparison operators
            BaseExpr::Eq => Box::new(EqExpr),
            BaseExpr::NotEq => Box::new(NotEqExpr),
            BaseExpr::Lt => Box::new(LtExpr),
            BaseExpr::LtEq => Box::new(LtEqExpr),
            BaseExpr::Gt => Box::new(GtExpr),
            BaseExpr::GtEq => Box::new(GtEqExpr),
            BaseExpr::IsDistinctFrom => Box::new(IsDistinctFromExpr),
            BaseExpr::IsNotDistinctFrom => Box::new(IsNotDistinctFromExpr),
            BaseExpr::Like => Box::new(LikeExpr),
            BaseExpr::ILike => Box::new(ILikeExpr),
            BaseExpr::NotLike => Box::new(NotLikeExpr),
            BaseExpr::NotILike => Box::new(NotILikeExpr),
            BaseExpr::RegexMatch => Box::new(RegexMatchExpr),
            BaseExpr::RegexIMatch => Box::new(RegexIMatchExpr),
            BaseExpr::RegexNotMatch => Box::new(RegexNotMatchExpr),
            BaseExpr::RegexNotIMatch => Box::new(RegexNotIMatchExpr),
            BaseExpr::CurrentDate => Box::new(CurrentDateExpr),
            BaseExpr::CurrentTime => Box::new(CurrentTimeExpr),
            BaseExpr::Now => Box::new(NowExpr),
            BaseExpr::CurrentTimestamp => Box::new(CurrentTimestampExpr),
            BaseExpr::ToChar => Box::new(ToCharExpr),
            BaseExpr::DateFormat => Box::new(DateFormatExpr),
            BaseExpr::ToDate => Box::new(ToDateExpr),
            BaseExpr::ToLocalTime => Box::new(ToLocalTimeExpr),
            BaseExpr::ToTimestamp => Box::new(ToTimestampExpr),
            BaseExpr::ToTimestampMicros => Box::new(ToTimestampMicrosExpr),
            BaseExpr::ToTimestampMillis => Box::new(ToTimestampMillisExpr),
            BaseExpr::ToTimestampNanos => Box::new(ToTimestampNanosExpr),
            BaseExpr::ToTimestampSeconds => Box::new(ToTimestampSecondsExpr),
            BaseExpr::ToUnixtime => Box::new(ToUnixtimeExpr),
            BaseExpr::Today => Box::new(TodayExpr),
        }
    }
}
pub trait BaseExprWithInfo {
    fn describe(&self) -> ExprWrapper;

    /// Builds the actual DataFusion expression from child expressions.
    /// This method encapsulates the construction logic for each expression type.
    fn build_expr(&self, child_exprs: &[Expr]) -> Expr;
}

/// Returns all available expressions that can be used in query generation
pub fn all_available_exprs() -> &'static [Arc<ExprWrapper>] {
    static AVAILABLE_EXPRS: LazyLock<Vec<Arc<ExprWrapper>>> = LazyLock::new(|| {
        BaseExpr::iter()
            .map(|expr| Arc::new(expr.to_impl().describe()))
            .collect()
    });

    &AVAILABLE_EXPRS
}

impl TypeGroup {
    pub fn pick_random_type(&self, rng: &mut StdRng) -> DataType {
        match self {
            TypeGroup::SameAsOutput => {
                panic!("SameAsOutput type needs to be resolved with the output type")
            }
            TypeGroup::Fixed(dt) => dt.clone(),
            TypeGroup::OneOf(dts) => dts[rng.random_range(0..dts.len())].clone(),
        }
    }
}
