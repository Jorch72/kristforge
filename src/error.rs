use ::ocl::error::Error as OclError;
use ::std::convert::From;
use ::std::fmt::{Display, Formatter, Error as FmtError};
use ::reqwest::Error as ReqError;

#[derive(Debug)]
pub enum KristforgeError {
	OclError(OclError),
	InvalidSelector(String),
	HttpError(ReqError),
	InvalidAddressLength(usize),
	NoDevicesSelected
}

impl From<OclError> for KristforgeError {
	fn from(e: OclError) -> KristforgeError {
		KristforgeError::OclError(e)
	}
}

impl From<ReqError> for KristforgeError {
	fn from(e: ReqError) -> KristforgeError {
		KristforgeError::HttpError(e)
	}
}

impl Display for KristforgeError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		use KristforgeError::*;

		match self {
			OclError(e) => write!(f, "OpenCL error: {}", e),
			InvalidSelector(s) => write!(f, "Invalid device selector: {}", s),
			HttpError(e) => write!(f, "Network error: {}", e),
			InvalidAddressLength(l) => write!(f, "Invalid address - expected length 10, got {}", l),
			NoDevicesSelected => write!(f, "No devices selected")
		}
	}
}
