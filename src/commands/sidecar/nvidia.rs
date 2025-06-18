use std::process::Command;
use std::str;

use quick_xml::de::from_str;
use serde::Deserialize;

pub fn get_nvidia_metrics() -> String {
    match Command::new("nvidia-smi").arg("-q").arg("-x").output() {
        Ok(output) => match str::from_utf8(&output.stdout) {
            Ok(out) => {
                let log: NvidiaSmiLog = from_str(out).unwrap();
                let gpu = log.gpu;

                let mut result = String::new();
                result.push_str(&format!(
                    "nvidia_temp {}",
                    get_first_word(&gpu.temperature.gpu_temp)
                ));
                result.push_str(&format!(
                    "\nnvidia_power_draw {}",
                    get_first_word(&gpu.power_readings.power_draw)
                ));
                result.push_str(&format!(
                    "\nnvidia_graphics_clock {}",
                    get_first_word(&gpu.clocks.graphics_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_sm_clock {}",
                    get_first_word(&gpu.clocks.sm_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_mem_clock {}",
                    get_first_word(&gpu.clocks.mem_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_video_clock {}",
                    get_first_word(&gpu.clocks.video_clock)
                ));
                result.push_str(&format!(
                    "\nnvidia_fan_speed {}",
                    get_first_word(&gpu.fan_speed)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_total {}",
                    get_first_word(&gpu.fb_memory_usage.total)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_free {}",
                    get_first_word(&gpu.fb_memory_usage.free)
                ));
                result.push_str(&format!(
                    "\nnvidia_fb_memory_used {}",
                    get_first_word(&gpu.fb_memory_usage.used)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_total {}",
                    get_first_word(&gpu.bar1_memory_usage.total)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_free {}",
                    get_first_word(&gpu.bar1_memory_usage.free)
                ));
                result.push_str(&format!(
                    "\nnvidia_bar1_memory_used {}",
                    get_first_word(&gpu.bar1_memory_usage.used)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_gpu {}",
                    get_first_word(&gpu.utilization.gpu_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_mem {}",
                    get_first_word(&gpu.utilization.memory_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_enc {}",
                    get_first_word(&gpu.utilization.encoder_util)
                ));
                result.push_str(&format!(
                    "\nnvidia_utilization_dec {}\n",
                    get_first_word(&gpu.utilization.decoder_util)
                ));
                result
            }
            Err(e) => {
                eprintln!("error parsing nvidia-smi stdout {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("error running nvidia-smi {}", e);
            panic!()
        }
    }
}

fn get_first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap()
}

#[derive(Debug, Deserialize, PartialEq)]
struct NvidiaSmiLog {
    gpu: Gpu,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Gpu {
    fan_speed: String,
    temperature: Temperature,
    power_readings: PowerReadings,
    clocks: Clocks,
    bar1_memory_usage: MemoryUsage,
    fb_memory_usage: MemoryUsage,
    utilization: Utilization,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Temperature {
    gpu_temp: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct PowerReadings {
    power_draw: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Clocks {
    graphics_clock: String,
    sm_clock: String,
    mem_clock: String,
    video_clock: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct MemoryUsage {
    total: String,
    used: String,
    free: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Utilization {
    gpu_util: String,
    memory_util: String,
    encoder_util: String,
    decoder_util: String,
}
