use ::ocl::{Device, DeviceType};
use ::ocl::error::Result as OclResult;
use ::ocl_extras::full_device_info::FullDeviceInfo;
use ::std::cmp::{max, min};
use ::std::ffi::CStr;

/// Kristforge-specific extensions for `ocl::Device`
pub trait DeviceExtensions {
	/// Get the desired vector width to use for mining with this device
	fn vector_width(&self) -> OclResult<u32>;

	/// Get the board name for this AMD device (CL_DEVICE_BOARD_NAME_AMD)
	fn amd_boardname(&self) -> OclResult<String>;

	/// Get the human-readable name for this device
	fn human_name(&self) -> OclResult<String>;
}

const CL_DEVICE_BOARD_NAME_AMD: u32 = 0x4038;

impl DeviceExtensions for Device {
	fn vector_width(&self) -> OclResult<u32> {
		Ok(if self.device_type()? == DeviceType::CPU {
			min(16, max(self.preferred_vector_width_char()?, self.native_vector_width_char()?))
		} else {
			1
		})
	}

	// TODO: handle errors properly
	fn amd_boardname(&self) -> OclResult<String> {
		// get the board name - null-terminated byte vector
		let bytes = self.info_raw(CL_DEVICE_BOARD_NAME_AMD)?;

		// convert bytes to &CStr
		let c_str = CStr::from_bytes_with_nul(&bytes[..]).unwrap();

		// convert to utf8 &str
		let str = c_str.to_str().unwrap();

		// finally convert to String and return
		Ok(str.to_string())
	}

	fn human_name(&self) -> OclResult<String> {
		Ok(if self.extensions()?.contains("cl_amd_device_attribute_query") {
			self.amd_boardname()?
		} else {
			self.name()?
		})
	}
}
