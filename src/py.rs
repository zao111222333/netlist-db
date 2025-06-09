use std::path::PathBuf;

use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python, types::PyAnyMethods as _,
};
use pyo3_stub_gen::{PyStubType, TypeInfo};

use crate::FileId;

impl PyStubType for FileId {
    #[inline]
    fn type_output() -> TypeInfo {
        <PathBuf>::type_output() | <(PathBuf, String)>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for FileId {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            FileId::Include { path } => path.into_pyobject(py),
            FileId::Section { path, section } => {
                (path, section).into_pyobject(py).map(Bound::into_any)
            }
        }
    }
}

impl FromPyObject<'_> for FileId {
    #[inline]
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok((path, section)) = ob.extract::<(PathBuf, String)>() {
            return Ok(Self::Section { path, section });
        }
        let path = ob.extract::<PathBuf>()?;
        Ok(Self::Include { path })
    }
}
