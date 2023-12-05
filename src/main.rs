// TODO: support CA33 command to take over metrics consumption
// TODO: support publishing to S-Miles cloud, too

mod inverter;
mod mqtt;
mod protos;

use crate::inverter::Inverter;
use crate::mqtt::{MetricCollector, Mqtt};
use crate::protos::hoymiles::RealData::HMSStateResponse;

use std::io::{Write};
use std::thread;
use std::time::Duration;

use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use protos::hoymiles::RealData;

#[derive(Parser)]
struct Cli {
    inverter_host: String,
    mqtt_broker_host: String,
    mqtt_username: Option<String>,
    mqtt_password: Option<String>,
    #[clap(default_value = "1883")]
    mqtt_broker_port: u16,
}

static REQUEST_DELAY: u64 = 60_000;

fn set_r_old_defaults(_r_old: &mut HMSStateResponse) {
   _r_old.dtu_sn = "0".to_string();
   _r_old.pv_current_power = -1;
   _r_old.pv_daily_yield = -1;
   _r_old.inverter_state[0].grid_voltage = -1;
   _r_old.inverter_state[0].grid_freq = -1;
   _r_old.inverter_state[0].temperature = -1;
   _r_old.port_state[0].pv_vol = -1;
   _r_old.port_state[0].pv_cur = -1;
   _r_old.port_state[0].pv_power = -1;
   _r_old.port_state[0].pv_energy_total = -1;
   _r_old.port_state[0].pv_daily_yield = -1;
   _r_old.port_state[1].pv_vol = -1;
   _r_old.port_state[1].pv_cur = -1;
   _r_old.port_state[1].pv_power = -1;
   _r_old.port_state[1].pv_energy_total = -1;
   _r_old.port_state[1].pv_daily_yield = -1;
}

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    // set up mqtt connection
    info!(
        "inverter: {}, mqtt broker {}",
        cli.inverter_host, cli.mqtt_broker_host
    );

    let mut inverter = Inverter::new(&cli.inverter_host);

    let mut mqtt = Mqtt::new(
        &cli.mqtt_broker_host,
        &cli.mqtt_username,
        &cli.mqtt_password,
        cli.mqtt_broker_port,
    );

    let mut _inv_state_old = inverter::NetworkState::Unknown;
    let mut _r_old: HMSStateResponse = HMSStateResponse::new();
    let mut first_start: bool = true;

    loop {
        if let Some(r) = inverter.update_state() {
            if first_start {
               _r_old = r.clone();
               set_r_old_defaults(&mut _r_old);
               first_start = false;
            }
            if inverter.state != _inv_state_old {
               mqtt.publish_gen((&"hms800wt2/pv_inv_state").to_string(),(&"Online").to_string());
               _inv_state_old = inverter.state;
            }
            mqtt.publish(&r, &_r_old);
            _r_old = r.clone();
        } else {
            if inverter.state != _inv_state_old {
               mqtt.publish_gen((&"hms800wt2/pv_inv_state").to_string(),(&"Offline").to_string());
               _inv_state_old = inverter.state;
            }
        }

        // TODO: this has to move into the Inverter struct in an async implementation
        thread::sleep(Duration::from_millis(REQUEST_DELAY));
    }
}


