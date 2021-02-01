use crate::{ext_interface::{DataStorage, Logger}, network::WebRTCReceive};
use crate::network::Network;
use crate::{
    config::{NodeConfig, NodeInfo},
    web_rtc::WebRTCSpawner,
    websocket::WebSocketConnection,
};
use std::sync::{Arc, Mutex};

// use self::logic::Logic;
// mod logic;

/// The node structure holds it all together. It is the main structure of the project.
pub struct Node {
    pub info: NodeInfo,
    pub nodes: Vec<NodeInfo>,
    pub network: Network,
    pub storage: Box<dyn DataStorage>,
    pub logger: Box<dyn Logger>,
    // logic: Arc<Mutex<Logic>>,
}

const CONFIG_NAME: &str = "nodeConfig";

impl Node {
    /// Create new node by loading the config from the storage.
    /// This also initializes the network and starts listening for
    /// new messages from the signalling server and from other nodes.
    /// The actual logic is handled in Logic.
    pub fn new(
        storage: Box<dyn DataStorage>,
        logger: Box<dyn Logger>,
        ws: Box<dyn WebSocketConnection>,
        web_rtc: WebRTCSpawner,
    ) -> Result<Node, String> {
        let config = NodeConfig::new(storage.load(CONFIG_NAME)?)?;
        storage.save(CONFIG_NAME, &config.to_string()?)?;
        logger.info(&format!("Starting node: {}", config.our_node.public));
        // let logic = Logic::new(config.our_node.clone());
        let log_clone = logger.clone();
        let web_rtc_rcv: WebRTCReceive = Arc::new(Mutex::new(Box::new(move |id, msg| {
            log_clone.info(&format!("id: {}, msg: {}", id, msg))
        })));
        let network = Network::new(
            ws,
            web_rtc,
            web_rtc_rcv,
            logger.clone(),
            config.our_node.clone(),
        );

        Ok(Node {
            info: config.our_node,
            storage,
            network,
            logger,
            nodes: vec![],
            // logic,
        })
    }

    /// TODO: this is only for development
    pub async fn clear(&self) {
        self.network.clear_nodes();
    }

    /// Requests a list of all connected nodes
    pub async fn list(&mut self) {
        self.network.update_node_list()
    }

    /// Pings all known nodes
    pub async fn ping(&mut self) {
        for node in &self.network.get_list() {
            self.logger.info(&format!("Contacting node {:?}", node));
            match self.network.send(&node.public, "ping".to_string()).await {
                Ok(_) => self.logger.info("Successfully sent ping"),
                Err(e) => self
                    .logger
                    .error(&format!("Error while sending ping: {:?}", e)),
            }
        }
    }
}
