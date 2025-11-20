use std::path::PathBuf;

use pyo3::{Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
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

impl FromPyObject<'_, '_> for FileId {
    type Error = PyErr;
    #[inline]
    fn extract(ob: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if let Ok(path_section) = ob.extract::<(PathBuf, String)>() {
            return Ok(path_section.into());
        }
        let path = ob.extract::<PathBuf>()?;
        Ok(path.into())
    }
}
