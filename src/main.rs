#[macro_use]
extern crate structopt;
extern crate reqwest;
extern crate ocl;
extern crate ocl_extras;
extern crate ascii_tree;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod error;
mod extensions;
mod selector;

use structopt::StructOpt;
use ocl::{Platform, Device, DeviceType};
use ocl::error::Error as OclError;
use std::convert::From;
use error::KristforgeError;
use extensions::DeviceExtensions;
use selector::DeviceSelector;

#[derive(Debug, StructOpt)]
#[structopt(name = "kristforge", about = "OpenCL accelerated krist miner")]
enum CliOpts {
	/// Show information about OpenCL hardware
	#[structopt(name = "info")]
	Info,

	/// Mine krist using OpenCL hardware
	#[structopt(name = "mine")]
	Mine {
		/// Enable additional logging
		#[structopt(short = "v", long = "verbose")]
		verbose: bool,

		/// Target kernel execution rate
		#[structopt(short = "r", long = "rate", default_value = "5")]
		rate: f32,

		/// The krist address to mine for
		#[structopt(name = "ADDRESS")]
		address: String,

		/// Selectors to specify which devices to use. If omitted, all devices will be used.
		#[structopt(name = "SELECTOR")]
		selectors: Vec<DeviceSelector>
	},
}

fn display_info() -> Result<(), KristforgeError> {
	use ascii_tree::Tree::*;
	use ascii_tree::write_tree;
	use ocl_extras::full_device_info::FullDeviceInfo;

	let mut platforms = vec![];

	for (i, p) in Platform::list().iter().enumerate() {
		let mut devices = vec![];

		for (j, d) in Device::list(p, Some(DeviceType::ALL))?.iter().enumerate() {
			let mut info = vec![];

			info.push(format!("Device {} [p{}d{}]", d.human_name()?, i, j));
			info.push(format!("Vector width: {}", d.vector_width()?));
			info.push(format!("Max clock speed: {} MHz", d.max_clock_frequency()?));
			info.push(format!("Max compute units: {}", d.max_compute_units()?));

			devices.push(Leaf(info));
		}

		platforms.push(Node(format!("Platform {} [p{}]", p.name()?, i), devices));
	}

	let mut out = String::new();
	write_tree(&mut out, &Node("OpenCL".to_string(), platforms));
	println!("{}", out);
	Ok(())
}

fn main() -> Result<(), KristforgeError> {
	use CliOpts::*;

	match CliOpts::from_args() {
		Info => display_info()?,
		Mine { address, selectors, rate, verbose } => {

		}
	};

	Ok(())
}

