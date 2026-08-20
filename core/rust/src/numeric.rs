use std::cmp::Ordering;
use std::str::FromStr;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::core::Value;
use crate::lang::hash::{canonical_decimal_str_hash, hash_double};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CanonicalInteger {
    Small(i64),
    Big(String),
}

impl CanonicalInteger {
    pub(crate) fn from_bigint(value: BigInt) -> Self {
        match value.to_i64() {
            Some(value) => Self::Small(value),
            None => Self::Big(value.to_string()),
        }
    }
}

pub(crate) fn parse_integer_digits(
    digits: &str,
    radix: u32,
    negative: bool,
) -> Option<CanonicalInteger> {
    let mut value = BigInt::parse_bytes(digits.as_bytes(), radix)?;
    if negative {
        value = -value;
    }
    Some(CanonicalInteger::from_bigint(value))
}

pub(crate) fn parse_big_integer(value: &str) -> Result<BigInt, String> {
    BigInt::from_str(value).map_err(|_| format!("invalid integer representation: {value}"))
}

pub(crate) fn canonical_big_integer(value: &str) -> Result<String, String> {
    Ok(parse_big_integer(value)?.to_string())
}

fn pow10(exponent: u64) -> Result<BigInt, String> {
    let exponent = u32::try_from(exponent).map_err(|_| "decimal scale is too large".to_string())?;
    Ok(BigInt::from(10u8).pow(exponent))
}

fn checked_scale_add(left: i64, right: i64) -> Result<i64, String> {
    left.checked_add(right)
        .ok_or_else(|| "decimal scale is out of range".to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExactDecimal {
    coefficient: BigInt,
    /// Number of digits to the right of the decimal point. The represented
    /// value is `coefficient * 10^-scale`.
    scale: i64,
}

impl ExactDecimal {
    pub(crate) fn new(coefficient: BigInt, scale: i64) -> Self {
        let mut value = Self { coefficient, scale };
        value.normalize();
        value
    }

    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let source = source.trim();
        if source.is_empty() {
            return Err("empty decimal".into());
        }
        let (negative, unsigned) = match source.as_bytes()[0] {
            b'+' => (false, &source[1..]),
            b'-' => (true, &source[1..]),
            _ => (false, source),
        };
        if unsigned.is_empty() {
            return Err(format!("invalid decimal: {source}"));
        }
        let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
            Some(index) => {
                if unsigned[index + 1..].is_empty() {
                    return Err(format!("invalid decimal exponent: {source}"));
                }
                let exponent = unsigned[index + 1..]
                    .parse::<i64>()
                    .map_err(|_| format!("invalid decimal exponent: {source}"))?;
                (&unsigned[..index], exponent)
            }
            None => (unsigned, 0),
        };
        let mut digits = String::with_capacity(mantissa.len());
        let mut fractional = 0i64;
        let mut saw_point = false;
        for ch in mantissa.chars() {
            match ch {
                '0'..='9' => {
                    digits.push(ch);
                    if saw_point {
                        fractional = fractional
                            .checked_add(1)
                            .ok_or_else(|| "decimal scale is out of range".to_string())?;
                    }
                }
                '.' if !saw_point => saw_point = true,
                _ => return Err(format!("invalid decimal: {source}")),
            }
        }
        if digits.is_empty() {
            return Err(format!("invalid decimal: {source}"));
        }
        let mut coefficient = BigInt::parse_bytes(digits.as_bytes(), 10)
            .ok_or_else(|| format!("invalid decimal: {source}"))?;
        if negative {
            coefficient = -coefficient;
        }
        let scale = fractional
            .checked_sub(exponent)
            .ok_or_else(|| "decimal scale is out of range".to_string())?;
        Ok(Self::new(coefficient, scale))
    }

    pub(crate) fn from_integer(value: BigInt) -> Self {
        Self::new(value, 0)
    }

    pub(crate) fn from_f64(value: f64) -> Result<Self, String> {
        if !value.is_finite() {
            return Err("non-finite double cannot be represented as an exact decimal".into());
        }
        Self::parse(&value.to_string())
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coefficient.is_zero()
    }

    pub(crate) fn is_negative(&self) -> bool {
        self.coefficient.is_negative()
    }

    pub(crate) fn abs(self) -> Self {
        Self::new(self.coefficient.abs(), self.scale)
    }

    pub(crate) fn negated(self) -> Self {
        Self::new(-self.coefficient, self.scale)
    }

    fn normalize(&mut self) {
        if self.coefficient.is_zero() {
            self.scale = 0;
            return;
        }
        while self.scale > i64::MIN && (&self.coefficient % 10u8).is_zero() {
            self.coefficient /= 10u8;
            self.scale -= 1;
        }
    }

    fn scaled_coefficient(&self, target_scale: i64) -> Result<BigInt, String> {
        let difference = target_scale
            .checked_sub(self.scale)
            .ok_or_else(|| "decimal scale is out of range".to_string())?;
        if difference < 0 {
            return Err("internal decimal scale error".into());
        }
        Ok(&self.coefficient * pow10(difference as u64)?)
    }

    pub(crate) fn add(&self, other: &Self) -> Result<Self, String> {
        let scale = self.scale.max(other.scale);
        Ok(Self::new(
            self.scaled_coefficient(scale)? + other.scaled_coefficient(scale)?,
            scale,
        ))
    }

    pub(crate) fn subtract(&self, other: &Self) -> Result<Self, String> {
        let scale = self.scale.max(other.scale);
        Ok(Self::new(
            self.scaled_coefficient(scale)? - other.scaled_coefficient(scale)?,
            scale,
        ))
    }

    pub(crate) fn multiply(&self, other: &Self) -> Result<Self, String> {
        Ok(Self::new(
            &self.coefficient * &other.coefficient,
            checked_scale_add(self.scale, other.scale)?,
        ))
    }

    pub(crate) fn compare(&self, other: &Self) -> Result<Ordering, String> {
        let scale = self.scale.max(other.scale);
        Ok(self
            .scaled_coefficient(scale)?
            .cmp(&other.scaled_coefficient(scale)?))
    }

    fn ratio(&self, other: &Self) -> Result<(BigInt, BigInt), String> {
        if other.coefficient.is_zero() {
            return Err("division by zero".into());
        }
        let mut numerator = self.coefficient.clone();
        let mut denominator = other.coefficient.clone();
        let exponent = other
            .scale
            .checked_sub(self.scale)
            .ok_or_else(|| "decimal scale is out of range".to_string())?;
        if exponent > 0 {
            numerator *= pow10(exponent as u64)?;
        } else if exponent < 0 {
            denominator *= pow10(exponent.unsigned_abs())?;
        }
        if denominator.is_negative() {
            numerator = -numerator;
            denominator = -denominator;
        }
        let gcd = numerator.gcd(&denominator);
        Ok((numerator / &gcd, denominator / gcd))
    }

    pub(crate) fn divide_exact(&self, other: &Self) -> Result<Self, String> {
        let (mut numerator, mut denominator) = self.ratio(other)?;
        let mut twos = 0u64;
        let mut fives = 0u64;
        while (&denominator % 2u8).is_zero() {
            denominator /= 2u8;
            twos += 1;
        }
        while (&denominator % 5u8).is_zero() {
            denominator /= 5u8;
            fives += 1;
        }
        if denominator != BigInt::one() {
            return Err("non-terminating decimal division".into());
        }
        let scale = twos.max(fives);
        if twos < scale {
            numerator *= BigInt::from(2u8).pow((scale - twos) as u32);
        }
        if fives < scale {
            numerator *= BigInt::from(5u8).pow((scale - fives) as u32);
        }
        Ok(Self::new(
            numerator,
            i64::try_from(scale).map_err(|_| "decimal scale is too large".to_string())?,
        ))
    }

    pub(crate) fn quotient(&self, other: &Self) -> Result<Self, String> {
        let (numerator, denominator) = self.ratio(other)?;
        Ok(Self::from_integer(numerator / denominator))
    }

    pub(crate) fn remainder(&self, other: &Self) -> Result<Self, String> {
        let quotient = self.quotient(other)?;
        self.subtract(&other.multiply(&quotient)?)
    }

    pub(crate) fn modulo(&self, other: &Self) -> Result<Self, String> {
        let remainder = self.remainder(other)?;
        if remainder.is_zero() || remainder.is_negative() == other.is_negative() {
            Ok(remainder)
        } else {
            remainder.add(other)
        }
    }

    pub(crate) fn to_storage_string(&self) -> Result<String, String> {
        if self.coefficient.is_zero() {
            return Ok("0".into());
        }
        let negative = self.coefficient.is_negative();
        let digits = self.coefficient.abs().to_string();
        let mut output = String::new();
        if negative {
            output.push('-');
        }
        if self.scale <= 0 {
            output.push_str(&digits);
            let zeroes = usize::try_from(self.scale.unsigned_abs())
                .map_err(|_| "decimal scale is too large".to_string())?;
            output.extend(std::iter::repeat('0').take(zeroes));
            return Ok(output);
        }
        let scale =
            usize::try_from(self.scale).map_err(|_| "decimal scale is too large".to_string())?;
        if scale >= digits.len() {
            output.push_str("0.");
            output.extend(std::iter::repeat('0').take(scale - digits.len()));
            output.push_str(&digits);
        } else {
            let split = digits.len() - scale;
            output.push_str(&digits[..split]);
            output.push('.');
            output.push_str(&digits[split..]);
        }
        Ok(output)
    }

    pub(crate) fn to_display_string(&self) -> Result<String, String> {
        let mut output = self.to_storage_string()?;
        if !output.contains('.') {
            output.push_str(".0");
        }
        Ok(output)
    }

    pub(crate) fn to_bigint_exact(&self) -> Option<BigInt> {
        if self.scale <= 0 {
            let multiplier = pow10(self.scale.unsigned_abs()).ok()?;
            return Some(&self.coefficient * multiplier);
        }
        let divisor = pow10(self.scale as u64).ok()?;
        let (quotient, remainder) = self.coefficient.div_rem(&divisor);
        remainder.is_zero().then_some(quotient)
    }

    pub(crate) fn to_bigint_truncating(&self) -> Option<BigInt> {
        if self.scale <= 0 {
            let multiplier = pow10(self.scale.unsigned_abs()).ok()?;
            return Some(&self.coefficient * multiplier);
        }
        let divisor = pow10(self.scale as u64).ok()?;
        Some(&self.coefficient / divisor)
    }

    pub(crate) fn to_i64_exact(&self) -> Option<i64> {
        self.to_bigint_exact()?.to_i64()
    }

    pub(crate) fn to_f64(&self) -> Result<f64, String> {
        let value = self
            .to_storage_string()?
            .parse::<f64>()
            .map_err(|_| "numeric value is outside double range".to_string())?;
        if !value.is_finite() {
            return Err("numeric value is outside double range".into());
        }
        Ok(value)
    }
}

pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    ExactDecimal::parse(value)?.to_storage_string()
}

pub(crate) fn display_decimal(value: &str) -> Result<String, String> {
    ExactDecimal::parse(value)?.to_display_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Modulo,
}

pub(crate) fn is_integer_value(value: &Value) -> bool {
    matches!(value, Value::Number(_) | Value::BigInteger(_))
}

pub(crate) fn is_numeric_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Number(_) | Value::BigInteger(_) | Value::Decimal(_) | Value::Float(_)
    )
}

pub(crate) fn integer_value(value: &Value) -> Result<BigInt, String> {
    match value {
        Value::Number(value) => Ok(BigInt::from(*value)),
        Value::BigInteger(value) => parse_big_integer(value),
        _ => Err("expected an integer".into()),
    }
}

pub(crate) fn exact_decimal_value(value: &Value) -> Result<ExactDecimal, String> {
    match value {
        Value::Number(value) => Ok(ExactDecimal::from_integer(BigInt::from(*value))),
        Value::BigInteger(value) => Ok(ExactDecimal::from_integer(parse_big_integer(value)?)),
        Value::Decimal(value) => ExactDecimal::parse(value),
        _ => Err("expected an exact number".into()),
    }
}

pub(crate) fn compact_integer(value: BigInt) -> Value {
    match CanonicalInteger::from_bigint(value) {
        CanonicalInteger::Small(value) => Value::Number(value),
        CanonicalInteger::Big(value) => Value::BigInteger(value),
    }
}

pub(crate) fn compact_decimal(value: ExactDecimal) -> Result<Value, String> {
    Ok(Value::Decimal(value.to_storage_string()?))
}

fn incompatible_decimal_float(left: &Value, right: &Value) -> bool {
    (matches!(left, Value::Decimal(_)) && matches!(right, Value::Float(_)))
        || (matches!(left, Value::Float(_)) && matches!(right, Value::Decimal(_)))
}

fn float_value(value: &Value) -> Result<f64, String> {
    match value {
        Value::Float(value) => Ok(*value),
        Value::Number(value) => Ok(*value as f64),
        Value::BigInteger(value) => parse_big_integer(value)?
            .to_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| "numeric value is outside double range".to_string()),
        Value::Decimal(value) => ExactDecimal::parse(value)?.to_f64(),
        _ => Err("expected a numeric value".into()),
    }
}

fn float_binary(op: ArithmeticOp, left: f64, right: f64) -> Result<Value, String> {
    if matches!(
        op,
        ArithmeticOp::Divide | ArithmeticOp::Remainder | ArithmeticOp::Modulo
    ) && right == 0.0
    {
        return Err("division by zero".into());
    }
    let value = match op {
        ArithmeticOp::Add => left + right,
        ArithmeticOp::Subtract => left - right,
        ArithmeticOp::Multiply => left * right,
        ArithmeticOp::Divide => left / right,
        ArithmeticOp::Remainder => left % right,
        ArithmeticOp::Modulo => {
            let remainder = left % right;
            if remainder == 0.0 || remainder.is_nan() {
                remainder
            } else if remainder.is_sign_negative() != right.is_sign_negative() {
                remainder + right
            } else {
                remainder
            }
        }
    };
    Ok(Value::Float(value))
}

fn integer_binary(op: ArithmeticOp, left: BigInt, right: BigInt) -> Result<Value, String> {
    if matches!(
        op,
        ArithmeticOp::Divide | ArithmeticOp::Remainder | ArithmeticOp::Modulo
    ) && right.is_zero()
    {
        return Err("division by zero".into());
    }
    let value = match op {
        ArithmeticOp::Add => left + right,
        ArithmeticOp::Subtract => left - right,
        ArithmeticOp::Multiply => left * right,
        ArithmeticOp::Divide => left / right,
        ArithmeticOp::Remainder => left % right,
        ArithmeticOp::Modulo => {
            let remainder = &left % &right;
            if remainder.is_zero() || remainder.is_negative() == right.is_negative() {
                remainder
            } else {
                remainder + right
            }
        }
    };
    Ok(compact_integer(value))
}

fn decimal_binary(
    op: ArithmeticOp,
    left: ExactDecimal,
    right: ExactDecimal,
) -> Result<Value, String> {
    let value = match op {
        ArithmeticOp::Add => left.add(&right)?,
        ArithmeticOp::Subtract => left.subtract(&right)?,
        ArithmeticOp::Multiply => left.multiply(&right)?,
        ArithmeticOp::Divide => left.divide_exact(&right)?,
        ArithmeticOp::Remainder => left.remainder(&right)?,
        ArithmeticOp::Modulo => left.modulo(&right)?,
    };
    compact_decimal(value)
}

pub(crate) fn numeric_binary(
    op: ArithmeticOp,
    left: &Value,
    right: &Value,
) -> Result<Value, String> {
    if !is_numeric_value(left) || !is_numeric_value(right) {
        return Err("expected numeric values".into());
    }
    if incompatible_decimal_float(left, right) {
        return Err(
            "decimal and binary floating-point values require explicit conversion with double"
                .into(),
        );
    }
    if matches!(left, Value::Float(_)) || matches!(right, Value::Float(_)) {
        return float_binary(op, float_value(left)?, float_value(right)?);
    }
    if matches!(left, Value::Decimal(_)) || matches!(right, Value::Decimal(_)) {
        return decimal_binary(op, exact_decimal_value(left)?, exact_decimal_value(right)?);
    }
    integer_binary(op, integer_value(left)?, integer_value(right)?)
}

pub(crate) fn numeric_quotient(left: &Value, right: &Value) -> Result<Value, String> {
    if !is_numeric_value(left) || !is_numeric_value(right) {
        return Err("quot expects numeric values".into());
    }
    if incompatible_decimal_float(left, right) {
        return Err(
            "decimal and binary floating-point values require explicit conversion with double"
                .into(),
        );
    }
    if matches!(left, Value::Float(_)) || matches!(right, Value::Float(_)) {
        let left = float_value(left)?;
        let right = float_value(right)?;
        if right == 0.0 {
            return Err("division by zero".into());
        }
        return Ok(Value::Float((left / right).trunc()));
    }
    if matches!(left, Value::Decimal(_)) || matches!(right, Value::Decimal(_)) {
        return compact_decimal(exact_decimal_value(left)?.quotient(&exact_decimal_value(right)?)?);
    }
    integer_binary(
        ArithmeticOp::Divide,
        integer_value(left)?,
        integer_value(right)?,
    )
}

fn finite_float_decimal(value: f64) -> Result<ExactDecimal, String> {
    ExactDecimal::from_f64(value)
}

pub(crate) fn numeric_compare(left: &Value, right: &Value) -> Result<Option<Ordering>, String> {
    if !is_numeric_value(left) || !is_numeric_value(right) {
        return Ok(None);
    }
    if let (Value::Float(left), Value::Float(right)) = (left, right) {
        if left.is_nan() || right.is_nan() {
            return Ok(None);
        }
        if left.is_infinite() || right.is_infinite() {
            return Ok(left.partial_cmp(right));
        }
        return Ok(Some(
            finite_float_decimal(*left)?.compare(&finite_float_decimal(*right)?)?,
        ));
    }
    if let Value::Float(float) = left {
        if float.is_nan() {
            return Ok(None);
        }
        if *float == f64::INFINITY {
            return Ok(Some(Ordering::Greater));
        }
        if *float == f64::NEG_INFINITY {
            return Ok(Some(Ordering::Less));
        }
        return Ok(Some(
            finite_float_decimal(*float)?.compare(&exact_decimal_value(right)?)?,
        ));
    }
    if let Value::Float(float) = right {
        if float.is_nan() {
            return Ok(None);
        }
        if *float == f64::INFINITY {
            return Ok(Some(Ordering::Less));
        }
        if *float == f64::NEG_INFINITY {
            return Ok(Some(Ordering::Greater));
        }
        return Ok(Some(
            exact_decimal_value(left)?.compare(&finite_float_decimal(*float)?)?,
        ));
    }
    Ok(Some(
        exact_decimal_value(left)?.compare(&exact_decimal_value(right)?)?,
    ))
}

pub(crate) fn numeric_equal(left: &Value, right: &Value) -> Option<bool> {
    if !is_numeric_value(left) || !is_numeric_value(right) {
        return None;
    }
    if matches!((left, right), (Value::Float(a), Value::Float(b)) if a.is_nan() && b.is_nan()) {
        return Some(true);
    }
    Some(matches!(
        numeric_compare(left, right),
        Ok(Some(Ordering::Equal))
    ))
}

pub(crate) fn numeric_total_compare(left: &Value, right: &Value) -> Option<Ordering> {
    if !is_numeric_value(left) || !is_numeric_value(right) {
        return None;
    }
    let left_nan = matches!(left, Value::Float(value) if value.is_nan());
    let right_nan = matches!(right, Value::Float(value) if value.is_nan());
    match (left_nan, right_nan) {
        (true, true) => return Some(Ordering::Equal),
        (true, false) => return Some(Ordering::Greater),
        (false, true) => return Some(Ordering::Less),
        (false, false) => {}
    }
    numeric_compare(left, right).ok().flatten()
}

pub(crate) fn numeric_hash(value: &Value) -> Option<i32> {
    Some(match value {
        Value::Number(value) => canonical_decimal_str_hash(&value.to_string()),
        Value::BigInteger(value) | Value::Decimal(value) => canonical_decimal_str_hash(value),
        Value::Float(value) => hash_double(*value),
        _ => return None,
    })
}

pub(crate) fn numeric_negate(value: &Value) -> Result<Value, String> {
    match value {
        Value::Number(value) => match value.checked_neg() {
            Some(value) => Ok(Value::Number(value)),
            None => Ok(Value::BigInteger(BigInt::from(*value).abs().to_string())),
        },
        Value::BigInteger(value) => Ok(compact_integer(-parse_big_integer(value)?)),
        Value::Decimal(value) => compact_decimal(ExactDecimal::parse(value)?.negated()),
        Value::Float(value) => Ok(Value::Float(-value)),
        _ => Err("expected a numeric value".into()),
    }
}

pub(crate) fn numeric_abs(value: &Value) -> Result<Value, String> {
    match value {
        Value::Number(value) => match value.checked_abs() {
            Some(value) => Ok(Value::Number(value)),
            None => Ok(Value::BigInteger(BigInt::from(*value).abs().to_string())),
        },
        Value::BigInteger(value) => Ok(compact_integer(parse_big_integer(value)?.abs())),
        Value::Decimal(value) => compact_decimal(ExactDecimal::parse(value)?.abs()),
        Value::Float(value) => Ok(Value::Float(value.abs())),
        _ => Err("expected a numeric value".into()),
    }
}

pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    numeric_binary(ArithmeticOp::Add, value, &Value::Number(delta))
}

pub(crate) fn bit_not(value: &Value) -> Result<Value, String> {
    Ok(compact_integer(!integer_value(value)?))
}

pub(crate) fn bit_binary(operation: &str, left: &Value, right: &Value) -> Result<Value, String> {
    let left = integer_value(left)?;
    let right = integer_value(right)?;
    let value = match operation {
        "bit-and" => left & right,
        "bit-or" => left | right,
        "bit-xor" => left ^ right,
        _ => return Err(format!("unknown bit operation: {operation}")),
    };
    Ok(compact_integer(value))
}

fn shift_distance(value: &Value) -> Result<usize, String> {
    let value = integer_value(value)?;
    if value.is_negative() {
        return Err("shift distance must be a non-negative integer".into());
    }
    value
        .to_usize()
        .ok_or_else(|| "shift distance is outside the host index range".to_string())
}

pub(crate) fn bit_shift(left: bool, value: &Value, distance: &Value) -> Result<Value, String> {
    let value = integer_value(value)?;
    let distance = shift_distance(distance)?;
    Ok(compact_integer(if left {
        value << distance
    } else {
        value >> distance
    }))
}

fn boundary_integer(value: &Value) -> Result<BigInt, String> {
    match value {
        Value::Number(value) => Ok(BigInt::from(*value)),
        Value::BigInteger(value) => parse_big_integer(value),
        Value::Decimal(value) => ExactDecimal::parse(value)?
            .to_bigint_exact()
            .ok_or_else(|| "decimal is not an exact integer".to_string()),
        Value::Float(value) if value.is_finite() && value.fract() == 0.0 => {
            ExactDecimal::from_f64(*value)?
                .to_bigint_exact()
                .ok_or_else(|| "floating-point value is not an exact integer".to_string())
        }
        Value::Float(_) => Err("floating-point value is not an exact integer".into()),
        _ => Err("expected a numeric value".into()),
    }
}

pub(crate) fn to_i64_exact(value: &Value) -> Result<i64, String> {
    boundary_integer(value)?
        .to_i64()
        .ok_or_else(|| "integer is outside signed 64-bit range".to_string())
}

pub(crate) fn to_i64_truncating(value: &Value) -> Result<i64, String> {
    let integer = match value {
        Value::Number(value) => return Ok(*value),
        Value::BigInteger(value) => parse_big_integer(value)?,
        Value::Decimal(value) => ExactDecimal::parse(value)?
            .to_bigint_truncating()
            .ok_or_else(|| "decimal is outside signed 64-bit range".to_string())?,
        Value::Float(value) if value.is_finite() => ExactDecimal::from_f64(*value)?
            .to_bigint_truncating()
            .ok_or_else(|| "floating-point value is outside signed 64-bit range".to_string())?,
        Value::Float(_) => return Err("floating-point value is not finite".into()),
        _ => return Err("expected a numeric value".into()),
    };
    integer
        .to_i64()
        .ok_or_else(|| "integer is outside signed 64-bit range".to_string())
}

pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    boundary_integer(value)?
        .to_i32()
        .ok_or_else(|| "integer is outside signed 32-bit range".to_string())
}

pub(crate) fn to_u16_exact(value: &Value) -> Result<u16, String> {
    boundary_integer(value)?
        .to_u16()
        .ok_or_else(|| "integer is outside unsigned 16-bit range".to_string())
}

pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    boundary_integer(value)?
        .to_u32()
        .ok_or_else(|| "integer is outside unsigned 32-bit range".to_string())
}

pub(crate) fn to_u64_exact(value: &Value) -> Result<u64, String> {
    boundary_integer(value)?
        .to_u64()
        .ok_or_else(|| "integer is outside unsigned 64-bit range".to_string())
}

pub(crate) fn to_usize_exact(value: &Value) -> Result<usize, String> {
    boundary_integer(value)?
        .to_usize()
        .ok_or_else(|| "integer is outside the host index range".to_string())
}

pub(crate) fn to_f64_explicit(value: &Value) -> Result<f64, String> {
    float_value(value)
}

#[cfg(test)]
mod tests {
    use super::{parse_integer_digits, CanonicalInteger, ExactDecimal};
    use crate::core::Value;
    use crate::lang::hash::JavaHash;
    use crate::lang::protocol::HashType;
    use std::cmp::Ordering;
    use std::collections::HashSet;

    #[test]
    fn canonicalizes_integer_and_decimal_text() {
        assert_eq!(
            parse_integer_digits("9223372036854775808", 10, false),
            Some(CanonicalInteger::Big("9223372036854775808".into()))
        );
        assert_eq!(
            ExactDecimal::parse("001.2300e2")
                .unwrap()
                .to_storage_string()
                .unwrap(),
            "123"
        );
        assert_eq!(
            ExactDecimal::parse("1.0")
                .unwrap()
                .to_display_string()
                .unwrap(),
            "1.0"
        );
    }

    #[test]
    fn decimal_arithmetic_is_exact() {
        let one = ExactDecimal::parse("1.0").unwrap();
        let eight = ExactDecimal::parse("8.0").unwrap();
        assert_eq!(
            one.divide_exact(&eight)
                .unwrap()
                .to_storage_string()
                .unwrap(),
            "0.125"
        );
        assert_eq!(
            one.divide_exact(&ExactDecimal::parse("3.0").unwrap())
                .unwrap_err(),
            "non-terminating decimal division"
        );
        assert_eq!(
            ExactDecimal::parse("1.00")
                .unwrap()
                .compare(&ExactDecimal::parse("1").unwrap())
                .unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn equal_numeric_representations_share_order_hash_and_keys() {
        let compact = Value::Number(42);
        let promoted = Value::BigInteger("42".into());
        let decimal = Value::Decimal("42.000".into());
        let floating = Value::Float(42.0);

        for value in [&promoted, &decimal, &floating] {
            assert_eq!(compact, *value);
            assert_eq!(compact.cmp(value), Ordering::Equal);
            assert_eq!(
                compact.java_hash(HashType::Rapid),
                value.java_hash(HashType::Rapid)
            );
        }

        let mut keys = HashSet::new();
        keys.insert(compact);
        assert!(keys.contains(&promoted));
        assert!(keys.contains(&decimal));
        assert!(keys.contains(&floating));
    }

    #[test]
    fn decimal_quotient_remainder_and_modulo_follow_integer_sign_rules() {
        let minus_five = ExactDecimal::parse("-5.0").unwrap();
        let three = ExactDecimal::parse("3.0").unwrap();
        assert_eq!(
            minus_five
                .quotient(&three)
                .unwrap()
                .to_storage_string()
                .unwrap(),
            "-1"
        );
        assert_eq!(
            minus_five
                .remainder(&three)
                .unwrap()
                .to_storage_string()
                .unwrap(),
            "-2"
        );
        assert_eq!(
            minus_five
                .modulo(&three)
                .unwrap()
                .to_storage_string()
                .unwrap(),
            "1"
        );
    }
}
