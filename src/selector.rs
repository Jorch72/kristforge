use ::std::str::FromStr;
use ::error::KristforgeError::{self, InvalidSelector};
use ::regex::Regex;
use ::ocl::{Device, Platform};
use ::ocl::error::Result as OclResult;
use ::ocl::flags::DeviceType;

#[derive(Debug, Copy, Clone)]
pub enum DeviceSelector {
	All,
	Platform(usize),
	Device(usize, usize)
}

impl DeviceSelector {
	pub fn select_devices<'a>(&self, pool: &'a Vec<(Platform, Vec<Device>)>) -> Vec<&'a Device> {
		use DeviceSelector::*;

		match self {
			All => pool.iter().flat_map(|(_, d)| d).collect(),
			Platform(p) => pool[*p].1.iter().map(|d| d).collect(),
			Device(p, i) => vec![&pool[*p].1[*i]]
		}
	}

	pub fn select_all(selectors: &Vec<Self>) -> OclResult<Vec<Device>> {
		let mut pool = vec![];

		for p in Platform::list() {
			pool.push((p, Device::list(p, Some(DeviceType::ALL))?));
		}

		let mut selected = vec![];

		for selector in selectors {
			for device in selector.select_devices(&pool) {
				if !selected.contains(device) {
					selected.push(*device);
				}
			}
		};

		Ok(selected)
	}
}

impl FromStr for DeviceSelector {
	type Err = KristforgeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::DeviceSelector::*;

		lazy_static! { static ref PLATFORM_PATTERN: Regex = Regex::new(r"^p(\d+)$").unwrap(); }
		lazy_static! { static ref DEVICE_PATTERN: Regex = Regex::new(r"^p(\d+)d(\d+)$").unwrap(); }

		match s.to_lowercase().as_str() {
			"a" => Ok(All),
			s if PLATFORM_PATTERN.is_match(s) => {
				Ok(Platform(PLATFORM_PATTERN.captures(s).unwrap().get(1).unwrap().as_str().parse().unwrap()))
			},
			s if DEVICE_PATTERN.is_match(s) => {
				let caps = DEVICE_PATTERN.captures(s).unwrap();

				Ok(Device(
					caps.get(1).unwrap().as_str().parse().unwrap(),
					caps.get(2).unwrap().as_str().parse().unwrap()
				))
			},
			_ => Err(InvalidSelector(s.to_string()))
		}
	}
}
