use lazy_static::lazy_static;
use regex::Regex;
use std::io::prelude::*;
use std::net::TcpStream;

pub fn get_hddtemp_metrics() -> String {
    // hddtemp service is listening on port 7634
    let mut stream =
        TcpStream::connect("127.0.0.1:7634").expect("could not connect to hddtemp service");

    lazy_static! {
        static ref HDDTEMP_PATTERN: Regex = Regex::new(
            r"\|CT1000MX500SSD4\|([0-9.]+)\|C\|.+\|Samsung SSD 860 EVO M.2\|([0-9.]+)\|C\|.+\|WDC WD20EFZX-68AWUN0\|([0-9.]+)\|C\|.+\|WDC WD20EFZX-68AWUN0\|([0-9.]+)\|C\|.*$"
        )
        .unwrap();
    }

    let mut v: Vec<u8> = Vec::new();
    stream
        .read_to_end(&mut v)
        .expect("hddtemp service did not respond properly");
    let res = String::from_utf8(v).expect("could not parse");

    if let Some(m) = HDDTEMP_PATTERN.captures(&res) {
        format!(
            "hddtemp_crucial_mx500_temp {}\nhddtemp_samsung_860_evo_temp {}\nhddtemp_wd_red_plus_1_temp {}\nhddtemp_wd_red_plus_2_temp {}\n",
            &m[1], &m[2], &m[3], &m[4],
        )
    } else {
        panic!("Could not parse hddtemp output")
    }
}
