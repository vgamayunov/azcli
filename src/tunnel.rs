use anyhow::{Context, Result, bail};
use base64::Engine;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::api_client::BastionClient;
use crate::models::BastionSku;

const WS_OPCODE_BINARY: u8 = 0x02;
const WS_OPCODE_CLOSE: u8 = 0x08;
const WS_OPCODE_PING: u8 = 0x09;

pub struct TunnelServer {
    listener: TcpListener,
    local_port: u16,
    bastion_endpoint: String,
    bastion_sku: BastionSku,
    resource_id: String,
    resource_port: u16,
    hostname: Option<String>,
    client: Arc<BastionClient>,
    last_token: tokio::sync::Mutex<Option<String>>,
    node_id: tokio::sync::Mutex<Option<String>>,
    active_connections: AtomicU64,
    shutdown: AtomicBool,
}

impl TunnelServer {
    pub async fn new(
        client: Arc<BastionClient>,
        local_port: u16,
        bastion_endpoint: String,
        bastion_sku: BastionSku,
        resource_id: String,
        resource_port: u16,
        hostname: Option<String>,
    ) -> Result<Arc<Self>> {
        let addr: SocketAddr = ([127, 0, 0, 1], local_port).into();
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind to {addr}"))?;

        let actual_port = listener.local_addr()?.port();
        info!("Tunnel server bound to 127.0.0.1:{actual_port}");

        Ok(Arc::new(Self {
            listener,
            local_port: actual_port,
            bastion_endpoint,
            bastion_sku,
            resource_id,
            resource_port,
            hostname,
            client,
            last_token: tokio::sync::Mutex::new(None),
            node_id: tokio::sync::Mutex::new(None),
            active_connections: AtomicU64::new(0),
            shutdown: AtomicBool::new(false),
        }))
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub async fn run(self: &Arc<Self>) -> Result<()> {
        info!("Tunnel server listening on port {}", self.local_port);

        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            let (stream, addr) = self.listener.accept().await?;
            info!("Accepted connection from {addr}");

            self.active_connections.fetch_add(1, Ordering::Relaxed);
            let server = Arc::clone(self);

            tokio::spawn(async move {
                if let Err(e) = server.handle_client(stream).await {
                    error!("Client handler error: {e:#}");
                }

                let remaining = server.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
                debug!("Connection closed. Active connections: {remaining}");

                if remaining == 0 {
                    debug!("Last connection closed, running auto-cleanup");
                    if let Err(e) = server.cleanup().await {
                        debug!("Auto-cleanup error (non-fatal): {e:#}");
                    }
                }
            });
        }

        Ok(())
    }

    async fn handle_client(self: &Arc<Self>, tcp_stream: TcpStream) -> Result<()> {
        let token_resp = {
            let last_token = self.last_token.lock().await;
            let node_id = self.node_id.lock().await;

            self.client
                .get_tunnel_token(
                    &self.bastion_endpoint,
                    &self.resource_id,
                    self.resource_port,
                    last_token.as_deref(),
                    self.hostname.as_deref(),
                    node_id.as_deref(),
                )
                .await?
        };

        {
            let mut last_token = self.last_token.lock().await;
            *last_token = Some(token_resp.auth_token.clone());
        }
        {
            let mut node_id = self.node_id.lock().await;
            *node_id = Some(token_resp.node_id.clone());
        }

        let ws_url = match self.bastion_sku {
            BastionSku::QuickConnect | BastionSku::Developer => {
                format!(
                    "wss://{}/omni/webtunnel/{}",
                    self.bastion_endpoint, token_resp.websocket_token
                )
            }
            _ => {
                format!(
                    "wss://{}/webtunnelv2/{}?X-Node-Id={}",
                    self.bastion_endpoint, token_resp.websocket_token, token_resp.node_id
                )
            }
        };

        debug!("Connecting WebSocket to {ws_url}");

        let tls_stream = self.connect_websocket(&ws_url).await?;

        debug!("WebSocket connected to bastion");

        let (ws_read, ws_write) = tokio::io::split(tls_stream);
        let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();

        let tcp_to_ws = async {
            let mut writer = WsWriter::new(ws_write);
            let mut buf = vec![0u8; 8192];
            loop {
                let n = tcp_read.read(&mut buf).await?;
                if n == 0 {
                    debug!("TCP client closed");
                    writer.write_close().await?;
                    break;
                }
                writer.write_binary(&buf[..n]).await?;
            }
            Ok::<_, anyhow::Error>(())
        };

        let ws_to_tcp = async {
            let mut reader = WsReader::new(ws_read);
            loop {
                match reader.read_frame().await? {
                    WsFrame::Binary(data) => {
                        tcp_write.write_all(&data).await?;
                    }
                    WsFrame::Ping(data) => {
                        debug!("WebSocket ping received ({} bytes)", data.len());
                    }
                    WsFrame::Close => {
                        debug!("WebSocket close frame received");
                        break;
                    }
                    WsFrame::Other(opcode) => {
                        debug!("WebSocket frame opcode={opcode:#x} (ignored)");
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        };

        tokio::select! {
            r = tcp_to_ws => {
                if let Err(e) = r {
                    debug!("TCP->WS ended: {e:#}");
                }
            }
            r = ws_to_tcp => {
                if let Err(e) = r {
                    debug!("WS->TCP ended: {e:#}");
                }
            }
        }

        Ok(())
    }

    async fn connect_websocket(
        &self,
        ws_url: &str,
    ) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
        let url: url::Url = ws_url.parse().context("Invalid WebSocket URL")?;
        let host = url
            .host_str()
            .context("WebSocket URL missing host")?
            .to_string();
        let port = url.port().unwrap_or(443);
        let path_and_query = match url.query() {
            Some(q) => format!("{}?{}", url.path(), q),
            None => url.path().to_string(),
        };

        let tcp = TcpStream::connect((&*host, port))
            .await
            .with_context(|| format!("TCP connect to {host}:{port}"))?;
        tcp.set_nodelay(true)?;

        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_config));
        let server_name = rustls::pki_types::ServerName::try_from(host.clone())
            .context("Invalid server name for TLS")?;
        let mut tls_stream = connector.connect(server_name, tcp).await?;

        let ws_key = {
            let bytes = *uuid::Uuid::new_v4().as_bytes();
            base64::engine::general_purpose::STANDARD.encode(bytes)
        };

        let upgrade_req = format!(
            "GET {path_and_query} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {ws_key}\r\nSec-WebSocket-Version: 13\r\nOrigin: https://{host}\r\n\r\n"
        );

        debug!("WS upgrade request path: {path_and_query}");
        tls_stream.write_all(upgrade_req.as_bytes()).await?;
        tls_stream.flush().await?;

        let mut response_buf = Vec::with_capacity(1024);
        let mut byte = [0u8; 1];
        loop {
            tls_stream.read_exact(&mut byte).await?;
            response_buf.push(byte[0]);
            let len = response_buf.len();
            if len >= 4
                && response_buf[len - 4..] == [b'\r', b'\n', b'\r', b'\n']
            {
                break;
            }
            if len > 8192 {
                bail!("WebSocket upgrade response too large");
            }
        }

        let response_str = String::from_utf8_lossy(&response_buf);
        let response_lower = response_str.to_lowercase();
        debug!("WS upgrade response:\n{}", response_str.trim());

        let status_line = response_str.lines().next().unwrap_or("");
        if !status_line.contains("101") {
            bail!("WebSocket upgrade rejected: {}", response_str.trim());
        }

        let has_upgrade = response_lower.contains("upgrade: websocket");
        let has_connection = response_lower.contains("connection: upgrade");
        if !has_upgrade || !has_connection {
            warn!(
                "WebSocket 101 response missing standard headers (upgrade={has_upgrade}, connection={has_connection}). \
                 Bastion server is known to omit these. Proceeding anyway."
            );
        }

        debug!("WebSocket handshake complete");

        Ok(tls_stream)
    }

    pub async fn cleanup(&self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        let last_token = self.last_token.lock().await;
        let node_id = self.node_id.lock().await;

        if let Some(token) = last_token.as_deref() {
            info!("Cleaning up tunnel session");
            self.client
                .delete_tunnel_token(&self.bastion_endpoint, token, node_id.as_deref())
                .await?;
        }

        Ok(())
    }
}

enum WsFrame {
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Close,
    Other(u8),
}

struct WsReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> WsReader<R> {
    fn new(inner: R) -> Self {
        Self { inner }
    }

    async fn read_frame(&mut self) -> Result<WsFrame> {
        let mut header = [0u8; 2];
        self.inner.read_exact(&mut header).await?;

        if header[0] == b'H' && header[1] == b'T' {
            let mut buf = Vec::with_capacity(4096);
            buf.extend_from_slice(&header);
            let mut tmp = vec![0u8; 4094];
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                self.inner.read(&mut tmp),
            )
            .await
            {
                Ok(Ok(n)) => buf.extend_from_slice(&tmp[..n]),
                _ => {}
            }
            let msg = String::from_utf8_lossy(&buf);
            bail!("Server sent HTTP response instead of WebSocket frame: {msg}");
        }

        let opcode = header[0] & 0x0F;
        let masked = header[1] & 0x80 != 0;
        let len_byte = header[1] & 0x7F;

        let payload_len: u64 = match len_byte {
            126 => {
                let mut buf = [0u8; 2];
                self.inner.read_exact(&mut buf).await?;
                u16::from_be_bytes(buf) as u64
            }
            127 => {
                let mut buf = [0u8; 8];
                self.inner.read_exact(&mut buf).await?;
                u64::from_be_bytes(buf)
            }
            n => n as u64,
        };

        let mask_key = if masked {
            let mut key = [0u8; 4];
            self.inner.read_exact(&mut key).await?;
            Some(key)
        } else {
            None
        };

        if payload_len > 16 * 1024 * 1024 {
            bail!("WebSocket frame too large: {payload_len} bytes");
        }

        let mut payload = vec![0u8; payload_len as usize];
        if !payload.is_empty() {
            self.inner.read_exact(&mut payload).await?;
        }

        if let Some(key) = mask_key {
            for (i, b) in payload.iter_mut().enumerate() {
                *b ^= key[i % 4];
            }
        }

        match opcode {
            WS_OPCODE_BINARY | 0x01 | 0x00 => Ok(WsFrame::Binary(payload)),
            WS_OPCODE_CLOSE => Ok(WsFrame::Close),
            WS_OPCODE_PING => Ok(WsFrame::Ping(payload)),
            _ => Ok(WsFrame::Other(opcode)),
        }
    }
}

struct WsWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> WsWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }

    async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<()> {
        let mask_key: [u8; 4] = rand_mask_key();

        let fin_opcode = 0x80 | opcode;
        let len = payload.len();

        if len < 126 {
            self.inner
                .write_all(&[fin_opcode, 0x80 | len as u8])
                .await?;
        } else if len <= 65535 {
            self.inner
                .write_all(&[fin_opcode, 0x80 | 126])
                .await?;
            self.inner
                .write_all(&(len as u16).to_be_bytes())
                .await?;
        } else {
            self.inner
                .write_all(&[fin_opcode, 0x80 | 127])
                .await?;
            self.inner
                .write_all(&(len as u64).to_be_bytes())
                .await?;
        }

        self.inner.write_all(&mask_key).await?;

        let mut masked = payload.to_vec();
        for (i, b) in masked.iter_mut().enumerate() {
            *b ^= mask_key[i % 4];
        }
        self.inner.write_all(&masked).await?;
        self.inner.flush().await?;

        Ok(())
    }

    async fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        self.write_frame(WS_OPCODE_BINARY, data).await
    }

    async fn write_close(&mut self) -> Result<()> {
        self.write_frame(WS_OPCODE_CLOSE, &[]).await
    }
}

fn rand_mask_key() -> [u8; 4] {
    let v = uuid::Uuid::new_v4();
    let bytes = v.as_bytes();
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}
