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
        network.set_node_info(config.our_node.clone());

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
            .map(|_| ())
    }

    pub async fn list(&mut self) {
        self.network.update_node_list()
    }

    pub async fn ping(&mut self) {
        for node in &self.network.get_list(){
            self.logger.info(&format!("Contacting node {:?}", node));
            match self.network.send(node.public.clone(), "ping".to_string()){
                Ok(_) => {self.logger.info("Successfully sent ping")}
                Err(e) => {self.logger.error(&format!("Error while sending ping: {:?}", e))}
            }
        }
    }
}
