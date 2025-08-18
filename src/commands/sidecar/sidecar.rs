use crate::commands::call::Host;
use crate::commands::sidecar::native::define_native_function;
use crate::commands::sidecar::wali::define_wali_function;
use wasmtime::Linker;

use clap::ValueEnum;

// #[derive(Parser)]
// #[clap(author, version, about)]
// #[clap(about = "Prometheus exporter for my desktop metrics")]
// struct Cli {
//     /// Port where the HTTP server is listening
//     #[arg(default_value_t = 7878, short = 'p')]
//     port: u32,

//     /// List of enabled exporters (all are enabled if none provided)
//     #[arg(value_enum, short = 'x')]
//     exporters: Vec<Exporter>,
// }

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[clap(rename_all = "snake_case")]
enum Exporter {
    Hddtemp,
    LmSensors,
    ProcMeminfo,
    ProcNetdev,
    ProcStat,
    Nvidia,
    NzxtAio,
    AmdRadeontop,
}

fn pku_node_exporter(_port: u32) {
    let exporters = vec![
        Exporter::Hddtemp,
        Exporter::LmSensors,
        Exporter::ProcMeminfo,
        Exporter::ProcNetdev,
        Exporter::ProcStat,
        Exporter::AmdRadeontop,
    ];

    // let mut lm_sensors = lm_sensors::get_lm_sensors();

    // if exporters.contains(&Exporter::LmSensors) {
    //     lm_sensors.init();
    // }

    // if exporters.contains(&Exporter::AmdRadeontop) {
    //     amd_radeontop::init();
    // }

    let mut result = String::new();
    if exporters.contains(&Exporter::NzxtAio) {
        result.push_str(&crate::commands::sidecar::nzxt_aio::get_aio_metrics());
    }
    if exporters.contains(&Exporter::LmSensors) {
        crate::commands::sidecar::lm_sensors::get_trace();
    }
    // if exporters.contains(&Exporter::Hddtemp) {
    //     result.push_str(&crate::commands::sidecar::hddtemp::get_hddtemp_metrics());
    // }
    if exporters.contains(&Exporter::Nvidia) {
        result.push_str(&crate::commands::sidecar::nvidia::get_nvidia_metrics());
    }
    if exporters.contains(&Exporter::ProcStat) {
        result.push_str(&crate::commands::sidecar::proc_stat::get_proc_stat());
    }
    if exporters.contains(&Exporter::ProcNetdev) {
        result.push_str(&crate::commands::sidecar::proc_netdev::get_proc_netdev());
    }
    if exporters.contains(&Exporter::ProcMeminfo) {
        result.push_str(&crate::commands::sidecar::proc_meminfo::get_proc_memifo());
    }

    if exporters.contains(&Exporter::AmdRadeontop) {
        result.push_str(&crate::commands::sidecar::amd_radeontop::get_radeontop_stats());
    }

    println!("{result}");
}

/// Define env function
pub(crate) fn define_sidecar_function(linker: &mut Linker<Host>) {
    linker
        .func_wrap("env", "PKUNodeExporter", pku_node_exporter)
        .unwrap();
    define_native_function(linker);
    define_wali_function(linker);
}
