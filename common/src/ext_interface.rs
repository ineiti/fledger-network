use async_trait::async_trait;

pub trait DataStorage {
    fn load(&self, key: &str) -> Result<String, String>;

    fn save(&self, key: &str, value: &str) -> Result<(), String>;
}

pub trait Logger: Send {
    fn info(&self, s: &str);
    fn warn(&self, s: &str);
    fn error(&self, s: &str);
    fn clone(&self) -> Box<dyn Logger>;
}

pub type WebRTCSpawner =
    Box<dyn Fn(WebRTCConnectionState) -> Result<Box<dyn WebRTCConnection>, String>>;

#[async_trait(?Send)]
pub trait WebRTCConnection {
    async fn call(
        &mut self,
        call: WebRTCMethod,
        input: Option<String>,
    ) -> Result<Option<String>, String>;
}

pub enum WebRTCMethod {
    MakeOffer,
    MakeAnswer,
    UseAnswer,
    WaitGathering,
    IceString,
    IcePut,
    MsgReceive,
    MsgSend,
    PrintStates,
}

/// What type of node this is
#[derive(PartialEq, Debug)]
pub enum WebRTCConnectionState {
    Initializer,
    Follower,
}
