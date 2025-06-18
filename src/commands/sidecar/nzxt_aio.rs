use hidapi::HidApi;

const KRAKEN_X52_VID: u16 = 0x1e71;
const KRAKEN_X52_PID: u16 = 0x170e;

pub fn get_aio_metrics() -> String {
    let api = HidApi::new().expect("could not initialize hidapi");
    let dev = api
        .open(KRAKEN_X52_VID, KRAKEN_X52_PID)
        .expect("could not connect to Kraken X52");
    dev.set_blocking_mode(true)
        .expect("could not set to blocking mode");

    // Approach copied from here
    // https://github.com/liquidctl/liquidctl/blob/678ac64451da80cf335d7848ddfdfbbf9adaa92d/liquidctl/driver/kraken2.py#L140-L142
    let mut buf = [0u8; 64];
    dev.read(&mut buf[..])
        .expect("Could not read from Kraken USB devices");
    let temp = buf[1] as f64 + buf[2] as f64 / 10.0;
    let fan_speed = (buf[3] as u64) << 8 | (buf[4] as u64);
    let pump_speed = (buf[5] as u64) << 8 | (buf[6] as u64);
    format!(
        "aio_liquid_temp {}\naio_fan_speed {}\naio_pump_speed {}\n",
        temp, fan_speed, pump_speed
    )
}
