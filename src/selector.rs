use ::std::str::FromStr;
use ::error::KristforgeError::{self, InvalidSelector};
use ::regex::Regex;

#[derive(Debug)]
pub enum DeviceSelector {
	All,
	Platform(usize),
	Device(usize, usize)
}

impl DeviceSelector {

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
