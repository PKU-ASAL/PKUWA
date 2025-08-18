mod sidecar;

mod amd_radeontop;
mod hddtemp;
mod lm_sensors;
mod nvidia;
mod nzxt_aio;
mod proc_meminfo;
mod proc_netdev;
mod proc_stat;
mod native;
mod wali;
mod companion;

pub(crate) use sidecar::*;
