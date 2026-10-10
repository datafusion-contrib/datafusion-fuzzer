use datafusion::arrow::datatypes::DataType;
use datafusion::config::ConfigOptions;
use datafusion::logical_expr::expr::ScalarFunction;
use datafusion::logical_expr::{BinaryExpr, Expr, Operator};
use datafusion_functions::datetime;

use super::expr_def::{BaseExpr, BaseExprWithInfo, ExprWrapper, TypeGroup};
use crate::common::{
    FuzzerDataType, get_available_data_types, get_numeric_data_types, get_time_data_types,
};

/// To add new expressions: Add a new variant to [`BaseExpr`] and then follow the pattern along.
/// - [x] Numeric Operators: +, -, *, /, %
/// - [x] Comparison Operators: =, !=, <, <=, >, >=, <=>, IS DISTINCT FROM, IS NOT DISTINCT FROM, ~, ~*, !~, !~*, ~~ (LIKE), ~~* (ILIKE), !~~ (NOT LIKE), !~~* (NOT ILIKE)
/// - [x] Logical Operators: AND, OR, NOT
/// - [x] Null Predicates: IS NULL, IS NOT NULL
/// - [ ] Bitwise Operators: &, |, #, >>, <<
/// - [ ] Other Operators: || (concat), @> (contains), <@ (contained by)
/// - [x] Time and Date Functions: current_date, current_time, current_timestamp, date_format, now, to_char, to_date, to_local_time, to_timestamp, to_timestamp_micros, to_timestamp_millis, to_timestamp_nanos, to_timestamp_seconds, to_unixtime, today
/// - [ ] Time and Date Functions (missing): date_bin, date_part, date_trunc, datepart, datetrunc, from_unixtime, make_date

// The following implementation includes several simplifications:
// The generation strategy aims to produce valid expressions with best effort;
// however, allowing the generation of invalid expressions can be beneficial if
// it simplifies the implementation.
//
// For date types, not all combinations in arithmetic expressions are valid:
// - Date32 + Date32 is invalid
// - Date32 + Interval is valid
// Nevertheless, we allow generating all combinations for simplicity.

// ========================
// Numeric Operators
// ========================
pub struct AddExpr;
impl BaseExprWithInfo for AddExpr {
    fn describe(&self) -> ExprWrapper {
        // Support both numeric and time types for addition
        let mut possible_return_types = get_numeric_data_types();
        possible_return_types.extend(get_time_data_types());

        let return_types: Vec<DataType> = possible_return_types
            .iter()
            .map(|ft| ft.to_datafusion_type())
            .collect();

        ExprWrapper {
            expr: BaseExpr::Add,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Plus,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct SubExpr;
impl BaseExprWithInfo for SubExpr {
    fn describe(&self) -> ExprWrapper {
        // Support both numeric and time types for subtraction
        let mut possible_return_types = get_numeric_data_types();
        possible_return_types.extend(get_time_data_types());

        let return_types: Vec<DataType> = possible_return_types
            .iter()
            .map(|ft| ft.to_datafusion_type())
            .collect();

        ExprWrapper {
            expr: BaseExpr::Sub,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Minus,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

// TODO(confirm): I think *, /, % are not available for time types?
pub struct MulExpr;
impl BaseExprWithInfo for MulExpr {
    fn describe(&self) -> ExprWrapper {
        let possible_return_types = get_numeric_data_types();

        let return_types: Vec<DataType> = possible_return_types
            .iter()
            .map(|ft| ft.to_datafusion_type())
            .collect();

        ExprWrapper {
            expr: BaseExpr::Mul,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Multiply,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct DivExpr;
impl BaseExprWithInfo for DivExpr {
    fn describe(&self) -> ExprWrapper {
        let possible_return_types = get_numeric_data_types();

        let return_types: Vec<DataType> = possible_return_types
            .iter()
            .map(|ft| ft.to_datafusion_type())
            .collect();

        ExprWrapper {
            expr: BaseExpr::Div,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Divide,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct ModExpr;
impl BaseExprWithInfo for ModExpr {
    fn describe(&self) -> ExprWrapper {
        let possible_return_types = get_numeric_data_types();

        let return_types: Vec<DataType> = possible_return_types
            .iter()
            .map(|ft| ft.to_datafusion_type())
            .collect();

        ExprWrapper {
            expr: BaseExpr::Mod,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Modulo,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

// ========================
// Logical Operators
// ========================
pub struct AndExpr;
impl BaseExprWithInfo for AndExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::And,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::And,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct OrExpr;
impl BaseExprWithInfo for OrExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Or,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput, TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Or,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct NotExpr;
impl BaseExprWithInfo for NotExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Not,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::SameAsOutput]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::Not(Box::new(child_exprs[0].clone()))
    }
}

// ========================
// Null Predicates
// ========================
pub struct IsNullExpr;
impl BaseExprWithInfo for IsNullExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::IsNull,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::OneOf(
                get_available_data_types()
                    .iter()
                    .map(|t| t.to_datafusion_type())
                    .collect(),
            )]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::IsNull(Box::new(child_exprs[0].clone()))
    }
}

pub struct IsNotNullExpr;
impl BaseExprWithInfo for IsNotNullExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::IsNotNull,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::OneOf(
                get_available_data_types()
                    .iter()
                    .map(|t| t.to_datafusion_type())
                    .collect(),
            )]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::IsNotNull(Box::new(child_exprs[0].clone()))
    }
}

// ========================
// Comparison Operators
// ========================
pub struct EqExpr;
impl BaseExprWithInfo for EqExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Eq,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Eq,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct NotEqExpr;
impl BaseExprWithInfo for NotEqExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::NotEq,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::NotEq,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct LtExpr;
impl BaseExprWithInfo for LtExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Lt,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Lt,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct LtEqExpr;
impl BaseExprWithInfo for LtEqExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::LtEq,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::LtEq,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct GtExpr;
impl BaseExprWithInfo for GtExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Gt,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::Gt,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct GtEqExpr;
impl BaseExprWithInfo for GtEqExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::GtEq,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .filter(|t| t.is_numeric() || t.is_time())
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::GtEq,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct IsDistinctFromExpr;
impl BaseExprWithInfo for IsDistinctFromExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::IsDistinctFrom,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::IsDistinctFrom,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct IsNotDistinctFromExpr;
impl BaseExprWithInfo for IsNotDistinctFromExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::IsNotDistinctFrom,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
                TypeGroup::OneOf(
                    get_available_data_types()
                        .iter()
                        .map(|t| t.to_datafusion_type())
                        .collect(),
                ),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::IsNotDistinctFrom,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

// Pattern matching operators
pub struct LikeExpr;
impl BaseExprWithInfo for LikeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Like,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::LikeMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct ILikeExpr;
impl BaseExprWithInfo for ILikeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ILike,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::ILikeMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct NotLikeExpr;
impl BaseExprWithInfo for NotLikeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::NotLike,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::NotLikeMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct NotILikeExpr;
impl BaseExprWithInfo for NotILikeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::NotILike,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::NotILikeMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

// Regex operators
pub struct RegexMatchExpr;
impl BaseExprWithInfo for RegexMatchExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::RegexMatch,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::RegexMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct RegexIMatchExpr;
impl BaseExprWithInfo for RegexIMatchExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::RegexIMatch,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::RegexIMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct RegexNotMatchExpr;
impl BaseExprWithInfo for RegexNotMatchExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::RegexNotMatch,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::RegexNotMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

pub struct RegexNotIMatchExpr;
impl BaseExprWithInfo for RegexNotIMatchExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Boolean.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::RegexNotIMatch,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(child_exprs[0].clone()),
            Operator::RegexNotIMatch,
            Box::new(child_exprs[1].clone()),
        ))
    }
}

// ========================
// Date/Time Functions
// ========================

/// Example usage (SQL):
///   select current_date();
pub struct CurrentDateExpr;
impl BaseExprWithInfo for CurrentDateExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Date32.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::CurrentDate,
            return_type: return_types,
            inferred_child_signature: vec![vec![]], // No arguments
        }
    }

    fn build_expr(&self, _child_exprs: &[Expr]) -> Expr {
        let current_date_udf = datetime::current_date();
        Expr::ScalarFunction(ScalarFunction::new_udf(
            current_date_udf,
            vec![], // No arguments for current_date
        ))
    }
}

/// Example usage (SQL):
///   select current_time();
pub struct CurrentTimeExpr;
impl BaseExprWithInfo for CurrentTimeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Time64Nanosecond.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::CurrentTime,
            return_type: return_types,
            inferred_child_signature: vec![vec![]], // No arguments
        }
    }

    fn build_expr(&self, _child_exprs: &[Expr]) -> Expr {
        let current_time_udf = datetime::current_time();
        Expr::ScalarFunction(ScalarFunction::new_udf(
            current_time_udf,
            vec![], // No arguments for current_time
        ))
    }
}

/// Example usage (SQL):
///   select now();
/// Returns the current UTC timestamp.
pub struct NowExpr;
impl BaseExprWithInfo for NowExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Now,
            return_type: return_types,
            inferred_child_signature: vec![vec![]], // No arguments
        }
    }

    fn build_expr(&self, _child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let now_udf = datetime::now(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            now_udf,
            vec![], // No arguments for now
        ))
    }
}

/// Example usage (SQL):
///   select current_timestamp();
/// Alias for now() - returns the current UTC timestamp.
pub struct CurrentTimestampExpr;
impl BaseExprWithInfo for CurrentTimestampExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::CurrentTimestamp,
            return_type: return_types,
            inferred_child_signature: vec![vec![]], // No arguments
        }
    }

    fn build_expr(&self, _child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let current_timestamp_udf = datetime::now(&config); // Same as now()
        Expr::ScalarFunction(ScalarFunction::new_udf(
            current_timestamp_udf,
            vec![], // No arguments for current_timestamp
        ))
    }
}

// ========================
// Date/Time Conversion Functions
// ========================

/// Example usage (SQL):
///   select to_char('2023-03-01'::date, '%d-%m-%Y');
/// Returns a string representation of a date, time, timestamp or duration based on a Chrono format.
pub struct ToCharExpr;
impl BaseExprWithInfo for ToCharExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::String.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToChar,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::Date32.to_datafusion_type(),
                    FuzzerDataType::Time64Nanosecond.to_datafusion_type(),
                    FuzzerDataType::Timestamp.to_datafusion_type(),
                    FuzzerDataType::IntervalMonthDayNano.to_datafusion_type(),
                ]),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let to_char_udf = datetime::to_char();
        Expr::ScalarFunction(ScalarFunction::new_udf(to_char_udf, child_exprs.to_vec()))
    }
}

/// Example usage (SQL):
///   select date_format('2023-03-01'::date, '%d-%m-%Y');
/// Alias for to_char() - returns a string representation of a date, time, timestamp or duration.
pub struct DateFormatExpr;
impl BaseExprWithInfo for DateFormatExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::String.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::DateFormat,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::Date32.to_datafusion_type(),
                    FuzzerDataType::Time64Nanosecond.to_datafusion_type(),
                    FuzzerDataType::Timestamp.to_datafusion_type(),
                    FuzzerDataType::IntervalMonthDayNano.to_datafusion_type(),
                ]),
                TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let date_format_udf = datetime::to_char(); // Same as to_char()
        Expr::ScalarFunction(ScalarFunction::new_udf(
            date_format_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_date('2023-01-31');
/// Converts a value to a date (YYYY-MM-DD).
pub struct ToDateExpr;
impl BaseExprWithInfo for ToDateExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Date32.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToDate,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::Float32.to_datafusion_type(),
                    FuzzerDataType::Float64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let to_date_udf = datetime::to_date();
        Expr::ScalarFunction(ScalarFunction::new_udf(to_date_udf, child_exprs.to_vec()))
    }
}

/// Example usage (SQL):
///   select to_local_time('2024-04-01T00:00:20Z'::timestamp);
/// Converts a timestamp with a timezone to a timestamp without a timezone.
pub struct ToLocalTimeExpr;
impl BaseExprWithInfo for ToLocalTimeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToLocalTime,
            return_type: return_types,
            inferred_child_signature: vec![vec![TypeGroup::OneOf(vec![
                FuzzerDataType::Timestamp.to_datafusion_type(),
            ])]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let to_local_time_udf = datetime::to_local_time();
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_local_time_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_timestamp('2023-01-31T09:26:56.123456789-05:00');
/// Converts a value to a timestamp (YYYY-MM-DDT00:00:00Z).
pub struct ToTimestampExpr;
impl BaseExprWithInfo for ToTimestampExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToTimestamp,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::UInt32.to_datafusion_type(),
                    FuzzerDataType::UInt64.to_datafusion_type(),
                    FuzzerDataType::Float32.to_datafusion_type(),
                    FuzzerDataType::Float64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let to_timestamp_udf = datetime::to_timestamp(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_timestamp_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_timestamp_micros('2023-01-31T09:26:56.123456789-05:00');
/// Converts a value to a timestamp with microsecond precision.
pub struct ToTimestampMicrosExpr;
impl BaseExprWithInfo for ToTimestampMicrosExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToTimestampMicros,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::UInt32.to_datafusion_type(),
                    FuzzerDataType::UInt64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let to_timestamp_micros_udf = datetime::to_timestamp_micros(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_timestamp_micros_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_timestamp_millis('2023-01-31T09:26:56.123456789-05:00');
/// Converts a value to a timestamp with millisecond precision.
pub struct ToTimestampMillisExpr;
impl BaseExprWithInfo for ToTimestampMillisExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToTimestampMillis,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::UInt32.to_datafusion_type(),
                    FuzzerDataType::UInt64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let to_timestamp_millis_udf = datetime::to_timestamp_millis(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_timestamp_millis_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_timestamp_nanos('2023-01-31T09:26:56.123456789-05:00');
/// Converts a value to a timestamp with nanosecond precision.
pub struct ToTimestampNanosExpr;
impl BaseExprWithInfo for ToTimestampNanosExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToTimestampNanos,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::UInt32.to_datafusion_type(),
                    FuzzerDataType::UInt64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let to_timestamp_nanos_udf = datetime::to_timestamp_nanos(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_timestamp_nanos_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_timestamp_seconds('2023-01-31T09:26:56.123456789-05:00');
/// Converts a value to a timestamp with second precision.
pub struct ToTimestampSecondsExpr;
impl BaseExprWithInfo for ToTimestampSecondsExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Timestamp.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToTimestampSeconds,
            return_type: return_types,
            inferred_child_signature: vec![vec![
                TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Int32.to_datafusion_type(),
                    FuzzerDataType::Int64.to_datafusion_type(),
                    FuzzerDataType::UInt32.to_datafusion_type(),
                    FuzzerDataType::UInt64.to_datafusion_type(),
                ]),
                TypeGroup::OneOf(vec![FuzzerDataType::String.to_datafusion_type()]),
            ]],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let config = ConfigOptions::new();
        let to_timestamp_seconds_udf = datetime::to_timestamp_seconds(&config);
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_timestamp_seconds_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select to_unixtime('2020-09-08T12:00:00+00:00');
/// Converts a value to seconds since the unix epoch (1970-01-01T00:00:00Z).
pub struct ToUnixtimeExpr;
impl BaseExprWithInfo for ToUnixtimeExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Int64.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::ToUnixtime,
            return_type: return_types,
            inferred_child_signature: vec![
                vec![TypeGroup::OneOf(vec![
                    FuzzerDataType::String.to_datafusion_type(),
                    FuzzerDataType::Date32.to_datafusion_type(),
                    FuzzerDataType::Timestamp.to_datafusion_type(),
                    FuzzerDataType::Float32.to_datafusion_type(),
                    FuzzerDataType::Float64.to_datafusion_type(),
                ])],
                vec![
                    TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                    TypeGroup::Fixed(FuzzerDataType::String.to_datafusion_type()),
                ],
            ],
        }
    }

    fn build_expr(&self, child_exprs: &[Expr]) -> Expr {
        let to_unixtime_udf = datetime::to_unixtime();
        Expr::ScalarFunction(ScalarFunction::new_udf(
            to_unixtime_udf,
            child_exprs.to_vec(),
        ))
    }
}

/// Example usage (SQL):
///   select today();
/// Alias of current_date().
pub struct TodayExpr;
impl BaseExprWithInfo for TodayExpr {
    fn describe(&self) -> ExprWrapper {
        let return_types = vec![FuzzerDataType::Date32.to_datafusion_type()];

        ExprWrapper {
            expr: BaseExpr::Today,
            return_type: return_types,
            inferred_child_signature: vec![vec![]], // No arguments
        }
    }

    fn build_expr(&self, _child_exprs: &[Expr]) -> Expr {
        let today_udf = datetime::current_date(); // Same as current_date()
        Expr::ScalarFunction(ScalarFunction::new_udf(
            today_udf,
            vec![], // No arguments for today
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{init_available_data_types, util::to_sql_string};
    use datafusion::prelude::col;

    #[test]
    fn not_and_null_predicates_return_boolean() {
        init_available_data_types();
        let exprs: [Box<dyn BaseExprWithInfo>; 3] = [
            Box::new(NotExpr),
            Box::new(IsNullExpr),
            Box::new(IsNotNullExpr),
        ];
        for expr in exprs {
            let wrapper = expr.describe();
            assert_eq!(wrapper.return_type, vec![DataType::Boolean]);
            assert_eq!(wrapper.inferred_child_signature.len(), 1);
            assert_eq!(wrapper.inferred_child_signature[0].len(), 1);
        }
    }

    #[test]
    fn not_and_null_predicates_unparse_to_sql() {
        let child = [col("v1")];
        assert_eq!(
            to_sql_string(&NotExpr.build_expr(&child)).unwrap(),
            "NOT v1"
        );
        assert_eq!(
            to_sql_string(&IsNullExpr.build_expr(&child)).unwrap(),
            "v1 IS NULL"
        );
        assert_eq!(
            to_sql_string(&IsNotNullExpr.build_expr(&child)).unwrap(),
            "v1 IS NOT NULL"
        );
    }
}
