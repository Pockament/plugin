use futures_util::stream::StreamExt;
use hyper::body::Bytes;
use hyper::client::Client;
use hyper::{Body, Request};
use rune::Module;

pub fn module() -> Module {
    let mut m = Module::with_crate("httpc");

    m.async_function(&["post"], post).unwrap();

    m
}

async fn post(uri: String, body: Vec<u8>) -> rune::Result<Vec<u8>> {
    let r = Client::new()
        .request(Request::post(uri).body(Body::from(body)).unwrap())
        .await;

    if r.is_err() {
        return Ok(Vec::new());
    }

    let mut collected = vec![];
    let rv: Vec<Result<Bytes, _>> = r.unwrap().into_body().collect().await;
    rv.into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .for_each(|v| collected.append(&mut v.to_vec()));

    Ok(collected)
}
