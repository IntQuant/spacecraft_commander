use std::{
    marker::PhantomData,
    result,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use aes_gcm::{
    aead::{consts::U12, Aead},
    Aes256Gcm, KeyInit, Nonce,
};
use rand_core::OsRng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use socket2::{SockRef, TcpKeepalive};
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc,
};
use tracing::{debug, info, trace, warn};
use x25519_dalek::{EphemeralSecret, PublicKey};

const MAX_PACKET_LEN: usize = 1024 * 1024 * 1024;

pub type Result<T> = result::Result<T, NetworkError>;

#[derive(Debug)]
pub enum MalformedReason {
    Serialization,
    Encryption,
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Received unexpected data from the other side, reason: {:?}", .0)]
    MalformedData(MalformedReason),
    #[error("IO error: {}", .0)]
    IOError(#[from] io::Error),
    #[error("{}", .0)]
    Misc(String),
    //#[error("Can't continue working in current state")]
    //ShouldStop,
}

impl NetworkError {
    pub fn new(desc: String) -> Self {
        Self::Misc(desc)
    }
}

impl From<bitcode::Error> for NetworkError {
    fn from(_value: bitcode::Error) -> Self {
        Self::MalformedData(MalformedReason::Serialization)
    }
}

struct NonceCounter(u128);

impl NonceCounter {
    fn next(&mut self) -> Nonce<U12> {
        let nonce_buf = self.0.to_le_bytes();
        self.0 += 2;
        *Nonce::from_slice(&nonce_buf[..12])
    }
}

pub struct WrappedReader<R> {
    reader: BufReader<OwnedReadHalf>,
    buffer: Vec<u8>,
    cipher: Arc<Aes256Gcm>,
    nonce: NonceCounter,
    phantom: PhantomData<R>,
}

pub struct WrappedWriter<S> {
    writer: BufWriter<OwnedWriteHalf>,
    cipher: Arc<Aes256Gcm>,
    nonce: NonceCounter,
    phantom: PhantomData<S>,
}

impl<R: DeserializeOwned> WrappedReader<R> {
    pub async fn read(&mut self) -> Result<(R, usize)> {
        let mut ret_read_len = 1;

        let len = self.reader.read_u8().await?;
        let len = if len == u8::MAX {
            ret_read_len += 4;
            self.reader.read_u32().await? as usize
        } else {
            len as usize
        };
        ret_read_len += len;

        debug!("Packet len: {len}");
        if len > MAX_PACKET_LEN {
            return Err(NetworkError::new(format!(
                "Encountered a packed with a len of {len}, which is too long"
            )));
        }
        if len > self.buffer.len() {
            self.buffer.resize(len, 0)
        }
        self.reader.read_exact(&mut self.buffer[..len]).await?;
        let nonce = self.nonce.next();
        let decrypted = self
            .cipher
            .decrypt(&nonce, &self.buffer[..len])
            .map_err(|_e| NetworkError::MalformedData(MalformedReason::Encryption))?;
        Ok((bitcode::deserialize(&decrypted)?, ret_read_len))
    }
}

impl<S: Serialize> WrappedWriter<S> {
    pub async fn write(&mut self, msg: S) -> Result<usize> {
        let mut ret_write_len = 1;
        let serialized = bitcode::serialize(&msg)?;

        let nonce = self.nonce.next();
        let msg = self.cipher.encrypt(&nonce, serialized.as_ref()).unwrap();
        let len = msg.len();
        if len < u8::MAX as _ {
            self.writer.write_u8(len as u8).await?;
        } else {
            ret_write_len += 4;
            self.writer.write_u8(u8::MAX).await?;
            self.writer.write_u32(len as u32).await?;
        }
        trace!("Sending {len} bytes");
        if len > MAX_PACKET_LEN {
            return Err(NetworkError::new(format!(
                "Encountered a packed with a len of {len}, which is too long"
            )));
        }
        self.writer.write_all(&msg).await?;
        self.writer.flush().await?;
        Ok(ret_write_len + msg.len())
    }
}

pub async fn wrap_and_split<R, S>(
    mut stream: TcpStream,
) -> Result<(WrappedReader<R>, WrappedWriter<S>)> {
    let keepalive = TcpKeepalive::new().with_time(Duration::from_secs(30));
    SockRef::from(&stream).set_tcp_keepalive(&keepalive)?;

    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let as_bytes = public.as_bytes();
    stream.write_all(as_bytes).await?;
    let mut their_public = [0u8; 32];
    stream.read_exact(&mut their_public).await?;
    let shared_secret = secret
        .diffie_hellman(&PublicKey::from(their_public))
        .to_bytes();
    let cipher = Arc::new(Aes256Gcm::new_from_slice(&shared_secret).unwrap());

    let (reader_nonce, writer_nonce) = if public.as_bytes() < &their_public {
        debug!("Lesser public key");
        (NonceCounter(0), NonceCounter(1))
    } else {
        debug!("Greater public key");
        (NonceCounter(1), NonceCounter(0))
    };

    let (reader, writer) = stream.into_split();
    Ok((
        WrappedReader {
            reader: BufReader::new(reader),
            buffer: vec![0; 1024],
            cipher: cipher.clone(),
            nonce: writer_nonce,
            phantom: PhantomData,
        },
        WrappedWriter {
            writer: BufWriter::new(writer),
            cipher,
            nonce: reader_nonce,
            phantom: PhantomData,
        },
    ))
}
struct RemoteEndpointShared {
    running: AtomicBool,
    received_count: AtomicUsize,
    sent_count: AtomicUsize,
    endpoint_id: EndpointId,
    created_at: Instant,
}

impl RemoteEndpointShared {
    fn new(peer_id: EndpointId) -> Arc<Self> {
        Arc::new(Self {
            running: AtomicBool::new(true),
            received_count: AtomicUsize::new(0),
            sent_count: AtomicUsize::new(0),
            endpoint_id: peer_id,
            created_at: Instant::now(),
        })
    }

    async fn reader_move_to_channel<R>(
        &self,
        mut reader: WrappedReader<R>,
        sender: mpsc::Sender<R>,
    ) -> Result<()>
    where
        R: DeserializeOwned + Send + 'static,
    {
        loop {
            let (val, size_read) = reader.read().await?;
            if sender.send(val).await.is_err() {
                return Ok(());
            }
            self.received_count.fetch_add(size_read, Ordering::Relaxed);
        }
    }

    async fn channel_move_to_writer<S>(
        &self,
        mut receiver: mpsc::Receiver<S>,
        mut writer: WrappedWriter<S>,
    ) -> Result<()>
    where
        S: Serialize + Send,
    {
        loop {
            let Some(msg) = receiver.recv().await else {
                return Ok(());
            };
            let size_write = writer.write(msg).await?;
            self.sent_count.fetch_add(size_write, Ordering::Relaxed);
        }
    }
}

impl Drop for RemoteEndpointShared {
    fn drop(&mut self) {
        let elapsed = self.created_at.elapsed().as_secs();
        let received = self.received_count.load(Ordering::Relaxed) as u64;
        let sent = self.sent_count.load(Ordering::Relaxed) as u64;
        info!(
            "Stats for endpoint {:?}: {} bytes received, {} bytes sent",
            self.endpoint_id, received, sent,
        );
        if elapsed > 0 {
            info!(
                "Existed for {} seconds, {} B/s rx, {} B/s tx",
                elapsed,
                received / elapsed,
                sent / elapsed
            );
        } else {
            info!("Existed for 0 seconds, no bandwidth info available");
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct EndpointId(pub u32);

pub struct RemoteEndpoint<R, S> {
    receiver: mpsc::Receiver<R>,
    sender: mpsc::Sender<S>,
    shared: Arc<RemoteEndpointShared>,
}

impl<R, S> RemoteEndpoint<R, S>
where
    R: DeserializeOwned + Send + 'static,
    S: Serialize + Send + 'static,
{
    pub async fn new(stream: TcpStream, endpoint_id: EndpointId) -> Result<Self> {
        if let Err(err) = stream.set_nodelay(true) {
            info!("Could not enable tcp nodelay: {}", err);
        }
        let shared = RemoteEndpointShared::new(endpoint_id);
        let (reader, writer) = wrap_and_split::<R, S>(stream).await?;
        let (sender, receiver) = mpsc::channel(128);
        let (sender_outbound, receiver_outbound) = mpsc::channel(128);
        tokio::spawn({
            let shared = shared.clone();
            async move {
                if let Err(err) = shared.reader_move_to_channel(reader, sender).await {
                    warn!("Connection error: {}", err);
                };
                info!("Connection lost [inbound]");
                shared.running.store(false, Ordering::Release)
            }
        });
        tokio::spawn({
            let shared = shared.clone();
            async move {
                if let Err(err) = shared
                    .channel_move_to_writer(receiver_outbound, writer)
                    .await
                {
                    warn!("Connection error: {}", err);
                }
                info!("Connection lost [outbound]");
                shared.running.store(false, Ordering::Release)
            }
        });
        Ok(Self {
            receiver,
            sender: sender_outbound,
            shared,
        })
    }

    pub fn try_recv(&mut self) -> Option<R> {
        self.receiver.try_recv().ok()
    }

    pub fn send(&mut self, msg: S) {
        self.sender.blocking_send(msg).ok();
    }

    pub fn is_connected(&self) -> bool {
        self.shared.running.load(Ordering::Relaxed)
    }

    pub fn has_space(&self) -> bool {
        self.sender.capacity() > 4
    }

    pub fn endpoint_id(&self) -> EndpointId {
        self.shared.endpoint_id
    }
}
