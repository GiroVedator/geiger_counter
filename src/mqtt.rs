use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::time::Duration;
use tokio::sync::mpsc;

const DEFAULT_BROKER_HOST: &str = "localhost";
const DEFAULT_BROKER_PORT: u16 = 1883;
const DEFAULT_CLIENT_ID: &str = "geiger_reader";
const TOPIC_RADIATION: &str = "geiger/radiation_nsv_h";
const TOPIC_CPM: &str = "geiger/cpm";

pub struct MqttClient {
    client: AsyncClient,
    event_tx: mpsc::Sender<MqttEvent>,
}

#[derive(Debug)]
pub enum MqttEvent {
    Message { topic: String, payload: String },
    Connected,
    Disconnected,
}

impl MqttClient {
    /// Creates a new MQTT client and connects to the broker.
    /// Returns the client and a receiver for incoming events.
    pub async fn new(
        broker_host: &str,
        broker_port: u16,
        client_id: &str,
    ) -> Result<(Self, mpsc::Receiver<MqttEvent>), rumqttc::ClientError> {
        let mut mqtt_options = MqttOptions::new(client_id, broker_host, broker_port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));
        mqtt_options.set_clean_session(true);

        let (client, mut event_loop) = AsyncClient::new(mqtt_options, 16);
        let (event_tx, event_rx) = mpsc::channel::<MqttEvent>(32);

        let tx = event_tx.clone();
        tokio::spawn(async move {
            loop {
                match event_loop.poll().await {
                    Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                        let _ = tx.send(MqttEvent::Connected).await;
                    }
                    Ok(Event::Incoming(Incoming::Publish(publish))) => {
                        let topic = publish.topic.clone();
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        let _ = tx.send(MqttEvent::Message { topic, payload }).await;
                    }
                    Err(_) => {
                        let _ = tx.send(MqttEvent::Disconnected).await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok((MqttClient { client, event_tx }, event_rx))
    }

    /// Creates a client with default connection settings read from environment variables,
    /// falling back to localhost:1883.
    pub async fn from_env() -> Result<(Self, mpsc::Receiver<MqttEvent>), rumqttc::ClientError> {
        let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| DEFAULT_BROKER_HOST.to_string());
        let port = std::env::var("MQTT_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_BROKER_PORT);
        let client_id =
            std::env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());

        Self::new(&host, port, &client_id).await
    }

    /// Publishes a radiation reading in nSv/h to the default topic.
    pub async fn publish_radiation_nsv_h(
        &self,
        value: f64,
    ) -> Result<(), rumqttc::ClientError> {
        let payload = format!("{:.3}", value);
        self.client
            .publish(TOPIC_RADIATION, QoS::AtLeastOnce, false, payload)
            .await
    }

    /// Publishes a CPM (counts per minute) reading to the default topic.
    pub async fn publish_cpm(&self, cpm: u32) -> Result<(), rumqttc::ClientError> {
        let payload = cpm.to_string();
        self.client
            .publish(TOPIC_CPM, QoS::AtLeastOnce, false, payload)
            .await
    }

    /// Publishes a raw payload to an arbitrary topic.
    pub async fn publish(
        &self,
        topic: &str,
        payload: &str,
        retain: bool,
    ) -> Result<(), rumqttc::ClientError> {
        self.client
            .publish(topic, QoS::AtLeastOnce, retain, payload)
            .await
    }

    /// Subscribes to a topic. Received messages arrive via the event receiver.
    pub async fn subscribe(&self, topic: &str) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await
    }

    /// Disconnects the client from the broker.
    pub async fn disconnect(&self) -> Result<(), rumqttc::ClientError> {
        self.client.disconnect().await
    }
}
