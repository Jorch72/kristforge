#[macro_use]
extern crate structopt;
extern crate reqwest;
extern crate ocl;
extern crate ocl_extras;
extern crate ascii_tree;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate crossbeam_channel;
extern crate ws;
extern crate rand;
extern crate sha2;
extern crate time;
extern crate indicatif;

mod error;
mod extensions;
mod selector;
mod common;
mod network;
mod miner;

use structopt::StructOpt;
use ocl::{Platform, Device, DeviceType};
use ocl::error::Error as OclError;
use error::KristforgeError;
use extensions::DeviceExtensions;
use selector::DeviceSelector;
use std::sync::{Mutex, Arc};
use miner::{MinerOpts, Miner, MinerCmd, MinerState, MinerCallback};
use rand::{random};
use common::Address;
use crossbeam_channel::{bounded, unbounded, Sender};
use common::Solution;
use indicatif::{MultiProgress, ProgressBar};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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
		rate: f64,

		/// The WS initiation URL
		#[structopt(short= "n", long = "node", default_value = "https://krist.ceriat.net/ws/start")]
		node: String,

		/// The krist address to mine for
		#[structopt(name = "ADDRESS")]
		address: Address,

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
	write_tree(&mut out, &Node("OpenCL".to_string(), platforms)).unwrap();
	println!("{}", out);
	Ok(())
}

struct MinerProgress {
	/// The progress bar to update with status
	bar: ProgressBar,

	/// The channel to send solutions on
	sol_tx: Sender<Solution>
}

impl MinerCallback for MinerProgress {
	fn on_state(&self, miner: &Miner, state: MinerState) {
		use MinerState::*;

		match state {
			Mining(hashrate) => self.bar.set_message(format!("Mining at {}", hashrate).as_str()),
			Solved(solution) => self.sol_tx.send(solution),
			Paused => self.bar.set_message("Idle"),
			Stopped => self.bar.finish_and_clear()
		}
	}
}

fn main() -> Result<(), KristforgeError> {
	use CliOpts::*;

	match CliOpts::from_args() {
		Info => display_info()?,
		Mine { address, selectors, rate, verbose, node } => {
			let devices = DeviceSelector::select_all(&selectors)?;

			let mut opts = MinerOpts { prefix: random(), vector_width: None, address, rate };
			let total_progress = MultiProgress::new();
			let (sol_tx, sol_rx) = bounded(0);
			let mut miner_channels = vec![];

			for d in devices {
				let progress = MinerProgress {
					bar: total_progress.add(ProgressBar::new_spinner()),
					sol_tx: sol_tx.clone()
				};

				progress.bar.set_prefix(d.human_name()?.as_str());

				let miner = Miner::new(d, &opts)?;

				// start miner and store channel
				miner_channels.push(miner.run_miner(progress)?);

				// increment prefix for next miner
				opts.prefix = opts.prefix.overflowing_add(1u8).0;
			}
		}
	};

	Ok(())
}
