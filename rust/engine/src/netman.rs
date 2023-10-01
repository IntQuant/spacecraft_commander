use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::mpsc,
    task::AbortHandle,
};
use tracing::{info, warn};

use crate::{
    enter_runtime, get_runtime,
    netman::net::EndpointId,
    universe::{OwnedUniverseEvent, PlayerID, Universe, UniverseEvent},
};

use self::{
    messages::{QueuedEvent, SentByClient, SentByServer},
    net::RemoteEndpoint,
};

mod messages;
mod net;

pub enum NetmanVariant {
    Client(Client),
    Server(Server),
}

pub struct Client {
    endpoint: CRemoteEndpoint,
    my_id: Option<PlayerID>,
}

type SRemoteEndpoint = RemoteEndpoint<SentByClient, SentByServer>;
type CRemoteEndpoint = RemoteEndpoint<SentByServer, SentByClient>;

pub struct Server {
    new_connections: mpsc::Receiver<SRemoteEndpoint>,
    endpoints: Vec<SRemoteEndpoint>,
    event_queue: VecDeque<QueuedEvent>,
    listener_task: AbortHandle,
    player_map: HashMap<EndpointId, PlayerID>,
    last_tick: Instant,
}

impl Client {
    fn process_events(&mut self, universe: &mut Universe) {
        while let Some(msg) = self.endpoint.try_recv() {
            match msg {
                SentByServer::SetUniverse(new_universe) => {
                    info!("Setting new universe...");
                    *universe = new_universe;
                    info!("Done!")
                }
                SentByServer::Event(QueuedEvent::StepUniverse) => universe.step(),
                SentByServer::Event(QueuedEvent::UniverseEvent(event)) => {
                    universe.process_event(event)
                }
                SentByServer::IdAssigned(id) => {
                    self.my_id = Some(id);
                    info!("Got id assigned: {:?}", id);
                    self.endpoint
                        .send(SentByClient::UniverseEvent(UniverseEvent::PlayerConnected));
                }
            }
        }
    }
}

const TICK_TIME: Duration = Duration::from_micros(16666);

impl Server {
    fn process_events(&mut self, universe: &mut Universe) {
        while let Ok(mut conn) = self.new_connections.try_recv() {
            conn.send(SentByServer::SetUniverse(universe.clone()));
            let new_id = PlayerID(conn.endpoint_id().0);
            conn.send(SentByServer::IdAssigned(new_id.clone()));
            self.player_map.insert(conn.endpoint_id(), new_id); // TODO proper auth
            self.endpoints.push(conn);
        }
        self.endpoints.retain(RemoteEndpoint::is_connected);

        // TODO round-robin
        for endpoint in &mut self.endpoints {
            while let Some(msg) = endpoint.try_recv() {
                match msg {
                    SentByClient::UniverseEvent(event) => {
                        if let Some(&player_id) = self.player_map.get(&endpoint.endpoint_id()) {
                            self.event_queue.push_back(QueuedEvent::UniverseEvent(
                                OwnedUniverseEvent { player_id, event },
                            ));
                        } else {
                            warn!(
                                "Can't map endpoint id ({:?}) to player id",
                                endpoint.endpoint_id()
                            );
                        }
                    }
                }
            }
        }

        let elapsed = self.last_tick.elapsed();
        if elapsed > TICK_TIME {
            self.last_tick += TICK_TIME;
            self.event_queue.push_back(QueuedEvent::StepUniverse);
        }
        if elapsed > TICK_TIME * 10 {
            self.last_tick = Instant::now();
            warn!("Lag detected - skipping 10+ ticks");
        }

        while !self.event_queue.is_empty() {
            let has_space = self.endpoints.iter().all(|x| x.has_space());
            if !has_space {
                break;
            }
            let msg = self.event_queue.pop_front().unwrap();

            for endpoint in &mut self.endpoints {
                endpoint.send(SentByServer::Event(msg.clone()));
            }
            match msg {
                QueuedEvent::UniverseEvent(event) => universe.process_event(event),
                QueuedEvent::StepUniverse => universe.step(),
            }
        }
    }
}

impl NetmanVariant {
    pub fn start_server() -> net::Result<Self> {
        info!("Starting server");
        let _rt = enter_runtime();
        // Бинд обычно происходит быстро, так что можно сделать синхронно
        let listener = get_runtime().block_on(TcpListener::bind("0.0.0.0:2300"))?;

        let (sender, new_connections) = mpsc::channel(1);

        let listener_task = tokio::spawn(async move {
            let mut next_id = 1;
            loop {
                let Ok((stream, addr)) = listener.accept().await else {
                    warn!("Unable to receive new connections");
                    break;
                };

                let endpoint_id = EndpointId(next_id);
                next_id += 1;
                info!("New connection from {}, id {:?}", addr, endpoint_id);
                match RemoteEndpoint::new(stream, endpoint_id).await {
                    Ok(endpoint) => {
                        info!("Connection from {:?} wrapped", endpoint_id);
                        sender.send(endpoint).await.expect("Channel active");
                    }
                    Err(err) => {
                        warn!("Could not wrap connection from {:?}: {}", endpoint_id, err)
                    }
                }
            }
        })
        .abort_handle();

        let mut event_queue = VecDeque::new();
        event_queue.push_back(QueuedEvent::UniverseEvent(OwnedUniverseEvent {
            player_id: PlayerID(0),
            event: UniverseEvent::PlayerConnected,
        }));

        Ok(Self::Server(Server {
            new_connections,
            listener_task,
            endpoints: Vec::new(),
            event_queue,
            player_map: HashMap::new(),
            last_tick: Instant::now(),
        }))
    }

    pub fn connect(addr: impl ToSocketAddrs) -> net::Result<Self> {
        let _rt = enter_runtime();
        // TODO проворачивать эту тему без блокировки
        let endpoint = get_runtime().block_on(async {
            let stream = TcpStream::connect(addr).await?;
            RemoteEndpoint::new(stream, EndpointId(0)).await
        })?;
        Ok(Self::Client(Client {
            endpoint,
            my_id: None,
        }))
    }

    pub fn process_events(&mut self, universe: &mut Universe) {
        match self {
            NetmanVariant::Client(client) => client.process_events(universe),
            NetmanVariant::Server(server) => server.process_events(universe),
        }
    }

    pub fn my_id(&self) -> Option<PlayerID> {
        match self {
            NetmanVariant::Client(client) => client.my_id,
            NetmanVariant::Server(_server) => Some(PlayerID(0)),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        info!("Aborting listener task");
        self.listener_task.abort();
    }
}
