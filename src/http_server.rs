use std::convert::Infallible;
use std::net::SocketAddr;

use futures_util::stream::StreamExt;
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use rune::runtime::SyncFunction;
use rune::{Module, Value};
use tokio::task::JoinHandle;

pub fn make_module() -> Module {
    let mut m = Module::new();

    m.async_function(&["https", "up"], up).unwrap();
    m.function(&["https", "down"], down).unwrap();
    m.function(&["https", "register"], register).unwrap();
    m.function(&["https", "unregister"], unregister).unwrap();
    m.function(&["https", "is_up"], is_up).unwrap();
    m.function(&["https", "is_registered"], is_registered)
        .unwrap();

    m
}

static mut EXEC: Option<SyncFunction> = None;

fn exec(body: Vec<u8>) -> Body {
    let body = body.into_iter().map(Value::from).collect::<Vec<_>>();
    let exec = unsafe { EXEC.as_ref() };

    match exec {
        None => Body::from(""),
        Some(func) => match func.call::<_, String>((body,)) {
            Err(_) => Body::from(""),
            Ok(o) => Body::from(o),
        },
    }
}

static mut HANDLE: Option<JoinHandle<()>> = None;

async fn up(port: u16) -> rune::Result<()> {
    let service = make_service_fn(|_| async move {
        Ok::<_, Infallible>(service_fn(|req: Request<Body>| async move {
            let mut collected = vec![];
            let rv: Vec<Result<Bytes, _>> = req.into_body().collect().await;
            rv.into_iter()
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap())
                .for_each(|v| collected.append(&mut v.to_vec()));

            Ok::<_, Infallible>(Response::new(exec(collected)))
        }))
    });

    let addr = SocketAddr::from(([0u8, 0, 0, 0], port));
    let server = Server::bind(&addr).serve(service);

    let server_handle = tokio::spawn(async move {
        let _ = server.await;
    });

    unsafe {
        HANDLE = Some(server_handle);
    }

    Ok(())
}

fn down() -> rune::Result<()> {
    if let Some(handle) = unsafe { HANDLE.take() } {
        handle.abort();
    }

    Ok(())
}

fn register(func: SyncFunction) -> rune::Result<()> {
    unsafe {
        EXEC = Some(func);
    }

    Ok(())
}

fn unregister() -> rune::Result<()> {
    unsafe {
        EXEC.take();
    }

    Ok(())
}

fn is_up() -> rune::Result<bool> { Ok(unsafe { HANDLE.is_some() }) }

fn is_registered() -> rune::Result<bool> { Ok(unsafe { EXEC.is_some() }) }
