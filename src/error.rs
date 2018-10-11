use ::ocl::error::{Error as OclError};
use ::std::convert::From;
use ::std::fmt::{Display, Formatter, Error as FmtError};

#[derive(Debug)]
pub enum KristforgeError {
	OclError(OclError),
	InvalidSelector(String)
}

impl From<OclError> for KristforgeError {
	fn from(e: OclError) -> KristforgeError {
		KristforgeError::OclError(e)
	}
}

impl Display for KristforgeError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		use KristforgeError::*;

		match self {
			OclError(e) => write!(f, "OpenCL error: {}", e),
			InvalidSelector(s) => write!(f, "Invalid device selector: {}", s)
		}
	}
}
