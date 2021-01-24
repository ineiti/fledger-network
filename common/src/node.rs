use crate::config::{NodeConfig, NodeInfo};
use crate::ext_interface::{DataStorage, Logger};
use crate::network::Network;use crate::types::U256;
use crate::web_rtc::Message;use crate::web_rtc::WebSocketMessage;

/// The node structure holds it all together. It is the main structure of the project.
pub struct Node {
    pub info: NodeInfo,
    pub nodes: Vec<NodeInfo>,
    pub network: Network,
    pub storage: Box<dyn DataStorage>,
    pub logger: Box<dyn Logger>,
}

const CONFIG_NAME: &str = "nodeConfig";

impl Node {
    /// Create new node by loading the config from the storage.
    /// If the storage is
    pub fn new(
        storage: Box<dyn DataStorage>,
        logger: Box<dyn Logger>,
        network: Network,
    ) -> Result<Node, String> {
        let config = NodeConfig::new(storage.load(CONFIG_NAME)?)?;
        logger.info("Config loaded");
        storage.save(CONFIG_NAME, &config.to_string()?)?;
        logger.info("Config saved");
        logger.info(&format!("Starting node: {}", config.our_node.public));

        Ok(Node {
            info: config.our_node,
            storage,
            network,
            logger,
            nodes: vec![],
        })
    }

    /// TODO: this is only for development
    pub async fn clear(&self) -> Result<(), String> {
        self.network
            .send(U256::rnd(), serde_json::to_string(&WebSocketMessage {
                msg: Message::ClearNodes,
            }).unwrap())
            .await
            .map(|_| ())
    }

    pub async fn connect(&mut self) {
        self.logger.info("Connecting to server");
        self.network
            .send(U256::rnd(), serde_json::to_string(&WebSocketMessage {
                msg: Message::Announce(self.info.clone()),
            }).unwrap())
            .await
            .unwrap();
        self.logger.info("Successfully announced at server");
    }
}
