use ::ocl::{Device, ProQue, Program, SpatialDims};
use ::ocl::error::Result as OclResult;
use ::ocl::flags::CommandQueueProperties as QueueProps;
use ::extensions::DeviceExtensions;
use ::std::fmt::{Display, Formatter, Error as FmtError};
use ::error::KristforgeError;
use ::rand::random;
use ::crossbeam_channel::{Sender, unbounded};
use ::common::{Target, Solution, Address};
use ::std::thread::{Builder as ThreadBuilder};
use ::std::cmp::{max, min};
use ::time::precise_time_ns;
use ::std::marker::Send;
use ::std::convert::From;

/// Options used when creating a miner
#[derive(Debug, Clone, PartialEq)]
pub struct MinerOpts {
	/// A value prepended to nonces during mining, to avoid multiple miners evaluating the same
	/// nonces.
	pub prefix: u8,

	/// The address to mine for
	pub address: Address,

	/// Target kernel execution rate (executions per second)
	pub rate: f64,

	/// A custom vector width. If `None`, one will be automatically chosen by the miner based on
	/// device information.
	pub vector_width: Option<u32>,
}

/// Hashing speed in hashes evaluated per second
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Hashrate(f64);

const SI_PREFIXES: &'static [&'static str] = &["", "K", "M", "G", "T"];

impl Display for Hashrate {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		let prefix_idx = min(self.0.log(1000.).floor() as usize, SI_PREFIXES.len());
		let prefix = SI_PREFIXES[prefix_idx];
		write!(f, "{:.2} {}H/s", self.0 / 1000f64.powi(prefix_idx as i32), prefix)
	}
}

/// Represents the current state of a miner
#[derive(Debug, Clone, PartialEq)]
pub enum MinerState {
	/// The miner has completed a cycle
	Mining(Hashrate),

	/// A solution has been found - the handler should implement submission
	Solved(Solution),

	/// The miner is paused
	Paused,

	/// The miner is permanently stopped and will no longer execute
	Stopped,
}

#[derive(Debug, Clone)]
pub enum MinerCmd {
	Mine(Target),
	Stop,
}

/// A miner, capable of mining krist using an OpenCL device
#[derive(Debug, Clone)]
pub struct Miner {
	pub device: Device,
	pub opts: MinerOpts,
	proque: ProQue,
}

pub trait MinerCallback {
	fn on_state(&self, miner: &Miner, state: MinerState);
}

/// OpenCL kernel source code
const MINER_SRC: &'static str = include_str!("kristforge.cl");

impl Miner {
	/// Create a miner using the given device and options.
	pub fn new(device: Device, opts: &MinerOpts) -> OclResult<Miner> {
		let vecsize = if let Some(v) = opts.vector_width {
			v
		} else {
			device.vector_width()?
		};

		Ok(Miner {
			device,
			opts: MinerOpts {
				vector_width: Some(vecsize),
				..*opts
			},
			proque: ProQue::builder()
				.prog_bldr(Program::builder()
					.src(MINER_SRC)
					.cmplr_def("VECSIZE", vecsize as i32)
					.clone())
				.queue_properties(QueueProps::PROFILING_ENABLE)
				.dims(SpatialDims::Unspecified)
				.build()?,
		})
	}

	/// Starts mining on another thread, using the given channels to submit solutions and status
	/// while returning a channel through which to send targets. Mining will stop and the thread
	/// will die when the returned target channel is closed. Note that this function moves `self` to
	/// the new mining thread, so make sure to clone it first if necessary.
	pub fn run_miner<C>(self, callback: C) -> OclResult<Sender<MinerCmd>>
		where C: 'static + Send + MinerCallback {
		let (target_tx, target_rx) = unbounded();

		ThreadBuilder::new()
			.name(format!("Miner on {}", self.device.human_name()?))
			.spawn(move || -> Result<(), KristforgeError> {
				// allocate array for solution output
				let mut solution = [0u8; Solution::NONCE_LEN];

				// allocate OpenCL buffers
				let addr_buf = self.proque.buffer_builder()
					.len(Address::LENGTH)
					.copy_host_slice(&self.opts.address.0)
					.build()?;

				let block_buf = self.proque.buffer_builder::<u8>()
					.len(Target::BLOCK_LEN)
					.build()?;

				let prefix_buf = self.proque.buffer_builder()
					.len(2)
					.copy_host_slice(&[self.opts.prefix, 0])
					.build()?;

				let solution_buf = self.proque.buffer_builder()
					.len(Solution::NONCE_LEN)
					.copy_host_slice(&solution)
					.build()?;

				// build kernel
				let kernel = self.proque.kernel_builder("kristMiner")
					.arg(&addr_buf)
					.arg(&block_buf)
					.arg(&prefix_buf)
					.arg_named("offset", 0i64)
					.arg_named("work", 0i64)
					.arg(&solution_buf)
					.build()?;

				// TODO: use usize for last_worksize?
				let mut last_worksize = 1u64;
				let mut last_duration = 0f64;

				loop {
					// block until we get a target
					let target = loop {
						match target_rx.recv() {
							Some(MinerCmd::Mine(t)) => break t,
							Some(MinerCmd::Stop) => {
								callback.on_state(&self, MinerState::Paused);
							}
							None => return Ok(())
						}
					};

					// update block and work
					block_buf.write(&target.prev_block[..]).enq()?;
					kernel.set_arg("work", target.work)?;

					let mut offset = 0u64;

					while target_rx.is_empty() {
						let started = precise_time_ns();

						// determine new worksize
						last_worksize = {
							let time_per_item = last_duration / (last_worksize as f64);
							let target_worksize = ((1. / self.opts.rate) / time_per_item) as u64;
							max(1, min(last_worksize * 16, target_worksize))
						};

						// update offset kernel arg
						kernel.set_arg("offset", offset)?;

						// execute kernel
						unsafe { kernel.cmd().global_work_size(last_worksize as usize).enq()? };

						// read solution buffer
						solution_buf.read(&mut solution[..]).enq()?;

						// TODO: use marker instead of iterating?
						if solution.iter().any(|v| *v != 0) {
							// found a solution!
							let solution = Solution { address: self.opts.address, nonce: solution, target };
							callback.on_state(&self, MinerState::Solved(solution));

							// zero the solution buffer
							solution_buf.cmd().fill(0u8, None).enq()?;
						}

						// update offset and last duration
						offset += last_worksize;
						last_duration = (precise_time_ns() - started) as f64 / 1e9;

						// send state
						callback.on_state(&self, MinerState::Mining(Hashrate(last_worksize as f64 / last_duration)))
					}
				};
			});

		Ok(target_tx)
	}
}

