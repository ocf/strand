use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientParams {
    pub group: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub client_params: ClientParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub kind: String,
    pub value: String,
}
