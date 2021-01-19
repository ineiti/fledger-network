use serde::{Deserialize, Serialize};

use crate::ext_interface::RestCaller;
use crate::ext_interface::RestMethod;
use crate::{config::NodeInfo, types::U256};

#[derive(Debug, Deserialize, Serialize)]
pub struct PostWebRTC {
    pub list_id: U256,
    pub node: NodeInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetListID {
    pub new_id: U256,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetWebRTC {
    pub list: Vec<NodeInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostConnectNode {
    pub node: U256,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetConnectNode {
    pub node: U256,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetConnectNodeReply {
    pub nodes: Vec<U256>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostPeerSend{
    pub to: U256,
    pub offer: Option<String>,
    pub candidate: Option<String>,
    pub answer: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostPeerReceive{
    pub from: U256,
    pub offer: Option<String>,
    pub candidate: Option<String>,
    pub answer: Option<String>,
}

pub struct RestClient {
    rest: Box<dyn RestCaller>,
}

impl RestClient {
    pub fn new(rest: Box<dyn RestCaller>) -> RestClient {
        RestClient { rest }
    }

    pub async fn clear_nodes(&self) -> Result<(), String> {
        self.rest
            .call(RestMethod::Delete, "clearNodes".to_string(), None)
            .await
            .map(|_| ())
    }

    pub async fn new_id(&mut self) -> Result<U256, String> {
        let reply = self
            .rest
            .call(RestMethod::Get, "newID".to_string(), None)
            .await?;
        let id: GetListID = serde_json::from_str(reply.as_str()).map_err(|e| e.to_string())?;
        Ok(id.new_id)
    }

    pub async fn add_id(&mut self, id: U256, ni: &NodeInfo) -> Result<(), String> {
        let pw = serde_json::to_string(&PostWebRTC {
            list_id: id,
            node: ni.clone(),
        })
        .map_err(|e| e.to_string())?;
        self.rest
            .call(RestMethod::Post, "addNode".to_string(), Some(pw))
            .await
            .map(|_| ())
    }

    pub async fn list_ids(&mut self) -> Result<Vec<NodeInfo>, String> {
        let reply = self
            .rest
            .call(RestMethod::Get, "listIDs".to_string(), None)
            .await?;
        let ids: GetWebRTC = serde_json::from_str(reply.as_str()).map_err(|e| e.to_string())?;
        Ok(ids.list)
    }
}
