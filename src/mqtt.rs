use std::{thread, time::Duration};

use crate::protos::hoymiles::RealData::HMSStateResponse;

use log::{warn};
use rumqttc::{Client, MqttOptions, QoS};

pub trait MetricCollector {
	fn publish_gen(&mut self,topic: String,payload: String);
    fn publish(&mut self, hms_state: &HMSStateResponse, hms_state_old: &HMSStateResponse);
}

pub struct Mqtt {
    client: Client,
}

impl Mqtt {
    pub fn new(
        host: &str,
        username: &Option<String>,
        password: &Option<String>,
        port: u16,
    ) -> Self {
        let mut mqttoptions = MqttOptions::new("hms800wt2-mqtt-publisher", host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        //parse the mqtt authentication options
        if let Some((username, password)) = match (username, password) {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(username), None) => Some((username.clone(), "".into())),
            (Some(username), Some(password)) => Some((username.clone(), password.clone())),
        } {
            mqttoptions.set_credentials(username, password);
        }

        let (client, mut connection) = Client::new(mqttoptions, 10);

        thread::spawn(move || {
            // keep polling the event loop to make sure outgoing messages get sent
            for _ in connection.iter() {}
        });

        Self { client }
    }
}

impl MetricCollector for Mqtt {

	fn publish_gen(&mut self, topic: String, payload: String) {
          if let Err(e) = self.client.publish(topic,QoS::AtMostOnce,true,payload) {
              warn!("mqtt gen error: {e}")
          }
	}
	
    fn publish(&mut self, hms_state: &HMSStateResponse, hms_state_old: &HMSStateResponse) {
        // debug!("{hms_state}");
        let topic_payload_pairs = [
            ("hms800wt2/pv_dtu_sn", (hms_state.dtu_sn).to_string(), (hms_state.dtu_sn != hms_state_old.dtu_sn)),
            ("hms800wt2/pv_current_power", (hms_state.pv_current_power as f32 / 10.).to_string(), (hms_state.pv_current_power != hms_state_old.pv_current_power)),
            ("hms800wt2/pv_daily_yield", (hms_state.pv_daily_yield as f32 / 1000.).to_string(), (hms_state.pv_daily_yield != hms_state_old.pv_daily_yield)),
            ("hms800wt2/pv_grid_voltage", (hms_state.inverter_state[0].grid_voltage as f32 / 10.).to_string(), (hms_state.inverter_state[0].grid_voltage != hms_state_old.inverter_state[0].grid_voltage)),
            ("hms800wt2/pv_grid_freq", (hms_state.inverter_state[0].grid_freq as f32 / 100.).to_string(), (hms_state.inverter_state[0].grid_freq != hms_state_old.inverter_state[0].grid_freq)),
            ("hms800wt2/pv_inv_temperature", (hms_state.inverter_state[0].temperature as f32 / 10.).to_string(), (hms_state.inverter_state[0].temperature != hms_state_old.inverter_state[0].temperature)),
            ("hms800wt2/pv_port1_voltage", (hms_state.port_state[0].pv_vol as f32 / 10.).to_string(), (hms_state.port_state[0].pv_vol != hms_state_old.port_state[0].pv_vol)),
            ("hms800wt2/pv_port1_curr", (hms_state.port_state[0].pv_cur as f32 / 100.).to_string(), (hms_state.port_state[0].pv_cur != hms_state_old.port_state[0].pv_cur)),
            ("hms800wt2/pv_port1_power", (hms_state.port_state[0].pv_power as f32 / 10.).to_string(), (hms_state.port_state[0].pv_power != hms_state_old.port_state[0].pv_power)),
            ("hms800wt2/pv_port1_energy", (hms_state.port_state[0].pv_energy_total as f32 / 10.).to_string(), (hms_state.port_state[0].pv_energy_total != hms_state_old.port_state[0].pv_energy_total)),
            ("hms800wt2/pv_port1_daily_yield", (hms_state.port_state[0].pv_daily_yield).to_string(), (hms_state.port_state[0].pv_daily_yield != hms_state_old.port_state[0].pv_daily_yield)),
            ("hms800wt2/pv_port2_voltage", (hms_state.port_state[1].pv_vol as f32 / 10.).to_string(), (hms_state.port_state[1].pv_vol != hms_state_old.port_state[1].pv_vol)),
            ("hms800wt2/pv_port2_curr", (hms_state.port_state[1].pv_cur as f32 / 100.).to_string(), (hms_state.port_state[1].pv_cur != hms_state_old.port_state[1].pv_cur)),
            ("hms800wt2/pv_port2_power", (hms_state.port_state[1].pv_power as f32 / 10.).to_string(), (hms_state.port_state[1].pv_power != hms_state_old.port_state[1].pv_power)),
            ("hms800wt2/pv_port2_energy", (hms_state.port_state[1].pv_energy_total as f32 / 10.).to_string(), (hms_state.port_state[1].pv_energy_total != hms_state_old.port_state[1].pv_energy_total)),
            ("hms800wt2/pv_port2_daily_yield", (hms_state.port_state[1].pv_daily_yield).to_string(), (hms_state.port_state[1].pv_daily_yield != hms_state_old.port_state[1].pv_daily_yield)),
        ];
        topic_payload_pairs
            .into_iter()
            .for_each(|(topic, payload, send)| {
				if send { 
                   if let Err(e) = self.client.publish(topic, QoS::AtMostOnce, true, payload) {
                       warn!("mqtt error: {e}")
				   }
                }
            });
    }
}
