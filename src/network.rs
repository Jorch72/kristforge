use ::reqwest::{Client, Result as ReqResult};
use ::ws::connect;

#[derive(Debug)]
struct WebsocketStartResponse {
	ok: bool,
	url: String,
	expires: f32
}
