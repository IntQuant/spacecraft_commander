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
    universe::{
        mcs::PlayerID, ui_events::UiEventCtx, OwnedUniverseEvent, Universe, UniverseEvent,
        TICK_TIME,
    },
};

use self::{
    messages::{QueuedEvent, SentByClient, SentByServer},
    net::RemoteEndpoint,
};

mod messages;
mod net;

/// Netmanager itself.
///
/// Ensures that each client's universe instance is being fed the same set of events.
pub enum NetmanVariant {
    Client(Client),
    Server(Server),
}

type SRemoteEndpoint = RemoteEndpoint<SentByClient, SentByServer>;
type CRemoteEndpoint = RemoteEndpoint<SentByServer, SentByClient>;

/// Implementation of client-side netmanager.
pub struct Client {
    endpoint: CRemoteEndpoint,
    my_id: Option<PlayerID>,

    event_queue: VecDeque<PartialEvent>,
    pending_steps: u32,
    last_step: Instant,
    latency_fix: i32,
    last_fix: Instant,
}

enum PartialEvent {
    Step,
    UniverseEvent(OwnedUniverseEvent),
}

/// Implementation of server-side netmanager.
pub struct Server {
    new_connections: mpsc::Receiver<SRemoteEndpoint>,
    endpoints: Vec<SRemoteEndpoint>,
    event_queue: VecDeque<QueuedEvent>,
    listener_task: AbortHandle,
    player_map: HashMap<EndpointId, PlayerID>,
    last_tick: Instant,
    last_late: Instant,
}

impl Client {
    fn process_events(&mut self, universe: &mut Universe) -> UiEventCtx {
        while let Some(msg) = self.endpoint.try_recv() {
            match msg {
                SentByServer::SetUniverse(new_universe) => {
                    self.last_step = Instant::now();
                    info!("Setting new universe...");
                    *universe = new_universe;
                    info!("Clearing queues...");
                    self.event_queue.clear();
                    self.pending_steps = 0;
                    info!("Done!")
                }
                SentByServer::Event(QueuedEvent::StepUniverse) => {
                    self.event_queue.push_back(PartialEvent::Step);
                    self.pending_steps += 1;
                }
                SentByServer::Event(QueuedEvent::UniverseEvent(event)) => self
                    .event_queue
                    .push_back(PartialEvent::UniverseEvent(event)),
                SentByServer::IdAssigned(id) => {
                    self.my_id = Some(id);
                    info!("Got id assigned: {:?}", id);
                    self.endpoint
                        .send(SentByClient::UniverseEvent(UniverseEvent::PlayerConnected));
                }
            }
        }
        let mut update_ctx = universe.update_ctx();
        while !self.event_queue.is_empty() && self.last_step.elapsed() > Duration::ZERO {
            match self.event_queue.pop_front().unwrap() {
                PartialEvent::Step => {
                    self.pending_steps -= 1;
                    self.last_step += TICK_TIME;
                    update_ctx.step();
                }
                PartialEvent::UniverseEvent(event) => update_ctx.process_event(event),
            }
        }
        if self.last_step.elapsed() > Duration::ZERO {
            self.last_step += Duration::from_millis(1);
            self.latency_fix += 1;
            self.last_fix = Instant::now();
            info!("Increasing latency_fix: {}", self.latency_fix);
        }
        if self.last_fix.elapsed() > Duration::from_secs(10) {
            self.latency_fix -= 1;
            self.last_step -= Duration::from_millis(1);
            info!("Decreasing latency_fix: {}", self.latency_fix);
            if self.latency_fix > 1000 {
                self.last_fix += Duration::from_millis(25);
            } else {
                self.last_fix += Duration::from_millis(100);
            }
        }
        update_ctx.evctx()
    }
}

impl Server {
    fn process_events(&mut self, universe: &mut Universe) -> UiEventCtx {
        while let Ok(mut conn) = self.new_connections.try_recv() {
            conn.send(SentByServer::SetUniverse(universe.clone()));
            let new_id = PlayerID(conn.endpoint_id().0);
            conn.send(SentByServer::IdAssigned(new_id));
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

        let mut update_ctx = universe.update_ctx();
        while !self.event_queue.is_empty() {
            let has_space = self.endpoints.iter().all(|x| x.has_space());
            if !has_space {
                if self.last_late.elapsed() > Duration::from_secs(5) {
                    self.last_late = Instant::now();
                    let late_ids = self
                        .endpoints
                        .iter()
                        .filter(|x| x.has_space())
                        .map(|x| x.endpoint_id())
                        .collect::<Vec<_>>();
                    warn!("Currently waiting on: {:?}", late_ids);
                }
                break;
            }
            let msg = self.event_queue.pop_front().unwrap();

            for endpoint in &mut self.endpoints {
                endpoint.send(SentByServer::Event(msg.clone()));
            }
            match msg {
                QueuedEvent::UniverseEvent(event) => update_ctx.process_event(event),
                QueuedEvent::StepUniverse => update_ctx.step(),
            }
        }
        update_ctx.evctx()
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
            last_late: Instant::now(),
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
            event_queue: Default::default(),
            pending_steps: 0,
            last_step: Instant::now(),
            latency_fix: 0,
            last_fix: Instant::now(),
        }))
    }

    pub fn process_events(&mut self, universe: &mut Universe) -> UiEventCtx {
        match self {
            NetmanVariant::Client(client) => client.process_events(universe),
            NetmanVariant::Server(server) => server.process_events(universe),
        }
    }

    pub fn emit_event(&mut self, event: UniverseEvent) -> bool {
        match self {
            NetmanVariant::Client(client) => {
                let has_space = client.endpoint.has_space();
                if has_space {
                    client.endpoint.send(SentByClient::UniverseEvent(event))
                }
                has_space
            }
            NetmanVariant::Server(server) => {
                server
                    .event_queue
                    .push_back(QueuedEvent::UniverseEvent(OwnedUniverseEvent {
                        player_id: PlayerID(0),
                        event,
                    }));
                true
            }
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
