use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

lazy_static! {
    static ref FIELD_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("ctxt", "procstat_ctxt");
        map.insert("btime", "procstat_btime_seconds");
        map.insert("processes", "procstat_processes");
        map.insert("procs_running", "procstat_procs_running");
        map.insert("procs_blocked", "procstat_procs_blocked");
        map.insert("procs_softirq", "procstat_procs_blocked");
        map
    };
    static ref CPU_FIELDS_MAP: &'static [&'static str] = &[
        "user",
        "nice",
        "system",
        "idle",
        "iowait",
        "irq",
        "softirq",
        "steal",
        "guest",
        "guestnice"
    ];
}

pub fn get_proc_stat() -> String {
    let file = File::open("/proc/stat").expect("cannot open /proc/stat");
    let lines = io::BufReader::new(file).lines();

    let mut result = String::new();

    for line in lines.map_while(Result::ok) {
        let mut iter = line.split_ascii_whitespace();
        let field_id = &iter.next().unwrap();
        if line.starts_with("cpu") {
            for (idx, item) in iter.enumerate() {
                if idx < 10 {
                    result.push_str(&format!(
                        "procstat_{}_{}_hz {}\n",
                        field_id, CPU_FIELDS_MAP[idx], item
                    ));
                }
            }
        } else if FIELD_MAP.contains_key(field_id) {
            result.push_str(&format!(
                "{} {}\n",
                FIELD_MAP[field_id],
                iter.next().unwrap()
            ));
        }
    }
    result
}
