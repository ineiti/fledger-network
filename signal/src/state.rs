use std::collections::HashMap;
use std::sync::Mutex;

use common::rest::PostPeerSend;
use common::types::U256;

use crate::node_list::NodeList;

pub struct ServerState {
    pub list: Mutex<NodeList>,
    pub peer: Mutex<HashMap<U256, HashMap<U256, PostPeerSend>>>,
    pub tokens: Mutex<HashMap<U256, U256>>,
}

impl ServerState {
    pub fn new() -> ServerState{
        let nl = NodeList::new();
        ServerState{
           list: Mutex::new(nl),
           peer: Mutex::new(HashMap::new()),
           tokens: Mutex::new(HashMap::new()),
       }
    }
    /// Merges a new PostPeerSend in the server structure.
    /// There should only be one token for any entry in 'from'.
    /// When sending an updated entry to 'to', tests need to show whether it's
    /// OK to update little by little.
    /// TODO: probably might need to delete the entry once it has been completely
    /// used to create a connection.
    pub fn merge(
        &self,
        from: &U256,
        token: &U256,
        update: &PostPeerSend,
    ) -> Result<PostPeerSend, String> {
        let tokens_lock = self.tokens.lock().unwrap();
        if let Some(st) = tokens_lock.get(from) {
            if st != token {
                return Err("wrong token for this id".to_string());
            }
        } else {
            return Err("this id is not registered".to_string());
        }

        let mut peer_lock = self.peer.lock().unwrap();
        let peer = match peer_lock.get(&from) {
            Some(p) => p.get(&update.to).unwrap_or(&update),
            None => {
                peer_lock.insert(from.clone(), HashMap::new());
                &update
            }
        };

        // Preference: new data from call, else already existing data, else None
        let new_peer = PostPeerSend{
            to: update.to.clone(),
            offer: update.offer.clone().or(peer.offer.clone()),
            candidate: update.candidate.clone().or(peer.candidate.clone()),
            answer: update.answer.clone().or(peer.answer.clone()),
        };

        peer_lock.get_mut(&from).unwrap().insert(update.to.clone(), new_peer.clone());

        Ok(new_peer.clone())
    }
}

#[cfg(test)]
mod tests {

}
