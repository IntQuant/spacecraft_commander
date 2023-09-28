use tokio::{net::TcpListener, sync::mpsc, task::AbortHandle};
use tracing::{info, warn};

use crate::{enter_runtime, get_runtime};

use self::{
    messages::{SentByClient, SentByServer},
    net::RemoteEndpoint,
};

mod messages;
mod net;

pub enum NetmanVariant {
    Client(Client),
    Server(Server),
}

pub struct Client {}

pub struct Server {
    new_connections: mpsc::Receiver<RemoteEndpoint<SentByClient, SentByServer>>,
    listener_task: AbortHandle,
}

impl NetmanVariant {
    pub fn start_server() -> net::Result<Self> {
        info!("Starting server");
        let _rt = enter_runtime();
        // Бинд обычно происходит быстро, так что можно сделать синхронно
        let listener = get_runtime().block_on(async { TcpListener::bind("0.0.0.0:4123").await })?;

        let (sender, new_connections) = mpsc::channel(1);

        let listener_task = tokio::spawn(async move {
            loop {
                let Ok((stream, addr)) = listener.accept().await else {
                    warn!("Unable to receive new connections");
                    break;
                };
                info!("New connection from {}", addr);

                match RemoteEndpoint::new(stream).await {
                    Ok(endpoint) => {
                        info!("Connection from {} wrapped", addr);
                        sender.send(endpoint).await.expect("Channel active");
                    }
                    Err(err) => {
                        warn!("Could not wrap connection from {}: {}", addr, err)
                    }
                }
            }
        })
        .abort_handle();

        Ok(Self::Server(Server {
            new_connections,
            listener_task,
        }))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        info!("Aborting listener task");
        self.listener_task.abort();
    }
}
