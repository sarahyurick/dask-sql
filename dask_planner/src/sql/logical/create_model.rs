use crate::sql::exceptions::py_type_err;
use crate::sql::logical;
use pyo3::prelude::*;

use datafusion_expr::logical_plan::UserDefinedLogicalNode;
use datafusion_expr::{Expr, LogicalPlan};

use fmt::Debug;
use std::{any::Any, fmt, sync::Arc};

use datafusion_common::DFSchemaRef;

#[derive(Clone)]
pub struct CreateModelPlanNode {
    pub model_name: String,
    pub input: LogicalPlan,
    pub or_replace: bool,
}

impl Debug for CreateModelPlanNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_for_explain(f)
    }
}

impl UserDefinedLogicalNode for CreateModelPlanNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![&self.input]
    }

    fn schema(&self) -> &DFSchemaRef {
        self.input.schema()
    }

    fn expressions(&self) -> Vec<Expr> {
        // there is no need to expose any expressions here since DataFusion would
        // not be able to do anything with expressions that are specific to
        // CREATE MODEL
        vec![]
    }

    fn fmt_for_explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CreateModel: model_name={}", self.model_name)
    }

    fn from_template(
        &self,
        _exprs: &[Expr],
        inputs: &[LogicalPlan],
    ) -> Arc<dyn UserDefinedLogicalNode> {
        assert_eq!(inputs.len(), 1, "input size inconsistent");
        Arc::new(CreateModelPlanNode {
            model_name: self.model_name.clone(),
            input: inputs[0].clone(),
            or_replace: self.or_replace,
        })
    }
}

#[pyclass(name = "CreateModel", module = "dask_planner", subclass)]
pub struct PyCreateModel {
    pub(crate) create_model: CreateModelPlanNode,
}

#[pymethods]
impl PyCreateModel {
    /// Creating a model requires that a subquery be passed to the CREATE MODEL
    /// statement to be used to gather the dataset which should be used for the
    /// model. This function returns that portion of the statement.
    #[pyo3(name = "getSelectQuery")]
    fn get_select_query(&self) -> PyResult<logical::PyLogicalPlan> {
        Ok(self.create_model.input.clone().into())
    }

    #[pyo3(name = "getModelName")]
    fn get_model_name(&self) -> PyResult<String> {
        Ok(self.create_model.model_name.clone())
    }

    #[pyo3(name = "getOrReplace")]
    pub fn get_or_replace(&self) -> PyResult<bool> {
        Ok(self.create_model.or_replace)
    }
}

impl TryFrom<logical::LogicalPlan> for PyCreateModel {
    type Error = PyErr;

    fn try_from(logical_plan: logical::LogicalPlan) -> Result<Self, Self::Error> {
        match logical_plan {
            logical::LogicalPlan::Extension(extension) => {
                if let Some(ext) = extension
                    .node
                    .as_any()
                    .downcast_ref::<CreateModelPlanNode>()
                {
                    Ok(PyCreateModel {
                        create_model: ext.clone(),
                    })
                } else {
                    Err(py_type_err("unexpected plan"))
                }
            }
            _ => Err(py_type_err("unexpected plan")),
        }
    }
}