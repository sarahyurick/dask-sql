
use pyo3::PyMappingProtocol;
use pyo3::{basic::CompareOp, prelude::*, PyNumberProtocol, PyObjectProtocol};
use std::convert::{From, Into};

use datafusion::arrow::datatypes::DataType;
use datafusion::logical_plan::{col, lit, Expr};

use datafusion::scalar::ScalarValue;

/// An PyExpr that can be used on a DataFrame
#[pyclass(name = "Expression", module = "datafusion", subclass)]
#[derive(Debug, Clone)]
pub struct PyExpr {
    pub expr: Expr,
}

impl From<PyExpr> for Expr {
    fn from(expr: PyExpr) -> Expr {
        expr.expr
    }
}

impl From<Expr> for PyExpr {
    fn from(expr: Expr) -> PyExpr {
        PyExpr { expr }
    }
}

#[pyproto]
impl PyNumberProtocol for PyExpr {
    fn __add__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok((lhs.expr + rhs.expr).into())
    }

    fn __sub__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok((lhs.expr - rhs.expr).into())
    }

    fn __truediv__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok((lhs.expr / rhs.expr).into())
    }

    fn __mul__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok((lhs.expr * rhs.expr).into())
    }

    fn __mod__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok(lhs.expr.modulus(rhs.expr).into())
    }

    fn __and__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok(lhs.expr.and(rhs.expr).into())
    }

    fn __or__(lhs: PyExpr, rhs: PyExpr) -> PyResult<PyExpr> {
        Ok(lhs.expr.or(rhs.expr).into())
    }

    fn __invert__(&self) -> PyResult<PyExpr> {
        Ok(self.expr.clone().not().into())
    }
}

#[pyproto]
impl PyObjectProtocol for PyExpr {
    fn __richcmp__(&self, other: PyExpr, op: CompareOp) -> PyExpr {
        let expr = match op {
            CompareOp::Lt => self.expr.clone().lt(other.expr),
            CompareOp::Le => self.expr.clone().lt_eq(other.expr),
            CompareOp::Eq => self.expr.clone().eq(other.expr),
            CompareOp::Ne => self.expr.clone().not_eq(other.expr),
            CompareOp::Gt => self.expr.clone().gt(other.expr),
            CompareOp::Ge => self.expr.clone().gt_eq(other.expr),
        };
        expr.into()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.expr))
    }
}

#[pymethods]
impl PyExpr {
    #[staticmethod]
    pub fn literal(value: ScalarValue) -> PyExpr {
        lit(value).into()
    }

    pub fn column_name(&self) -> String {
        match &self.expr {
            Expr::Alias{ .. } => { println!("Alias!"); String::from("alias") },
            Expr::Column(column) => { println!("Column: {:?}", &column.name); column.name.clone() },
            _ => panic!("Nothing found!!!")
        }
    }

    #[staticmethod]
    pub fn column(value: &str) -> PyExpr {
        col(value).into()
    }

    /// assign a name to the PyExpr
    pub fn alias(&self, name: &str) -> PyExpr {
        self.expr.clone().alias(name).into()
    }

    /// Create a sort PyExpr from an existing PyExpr.
    #[args(ascending = true, nulls_first = true)]
    pub fn sort(&self, ascending: bool, nulls_first: bool) -> PyExpr {
        self.expr.clone().sort(ascending, nulls_first).into()
    }

    pub fn is_null(&self) -> PyExpr {
        self.expr.clone().is_null().into()
    }

    pub fn cast(&self, to: DataType) -> PyExpr {
        // self.expr.cast_to() requires DFSchema to validate that the cast
        // is supported, omit that for now
        let expr = Expr::Cast {
            expr: Box::new(self.expr.clone()),
            data_type: to,
        };
        expr.into()
    }
}

#[pyproto]
impl PyMappingProtocol for PyExpr {
    fn __getitem__(&self, key: &str) -> PyResult<PyExpr> {
        Ok(Expr::GetIndexedField {
            expr: Box::new(self.expr.clone()),
            key: ScalarValue::Utf8(Some(key.to_string())),
        }
        .into())
    }
}