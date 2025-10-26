use actix_web::dev::ServerHandle;
use actix_web::middleware::Compress;
use actix_web::{App, HttpServer, rt, web};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::CertificateDer;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use std::time::Duration;

const LISTEN_ADDR: &str = "0.0.0.0";

/// Configure TLS options
pub enum TlsConfig {
    /// Auto generated self signed for testing http2/ssl
    SelfSigned { hosts: Vec<String> },
    /// Provided cert for production ssl and http2 alpn support
    Provided {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

/// Configures server limit options
pub struct ServerConfig {
    /// Port to attach http listener to. If none, will not accept plain http traffic
    pub http_port: Option<u16>,
    /// Port to attach https listener to. If none, will default to 443 if a tls config
    /// is provided. If None and no tls config, then no SSL listener will be configured.
    pub ssl_port: Option<u16>,
    /// Cert config required if ssl_port is set
    pub tls: Option<TlsConfig>,
    pub tcp_backlog: Option<u32>,
    pub max_conns: Option<usize>,
    pub threads: Option<usize>,
    /// Number of concurrent TLS handshakes allowed to be in progress, per worker (thread)
    /// Example default is 512, so on a 16 cpu server 512*16=8096 allowed tls conns
    /// being established at the same time.
    pub tls_rate_per_worker: Option<usize>,
}

/// Instance of an HTTP(S) server
pub struct Server {
    handle: ServerHandle,
}

impl Server {
    fn build_tls(cfg: TlsConfig) -> Result<rustls::ServerConfig, std::io::Error> {
        match cfg {
            TlsConfig::Provided {
                cert_path,
                key_path,
            } => {
                let cert_file = &mut BufReader::new(File::open(cert_path)?);
                let key_file = &mut BufReader::new(File::open(key_path)?);

                let cert_chain: Vec<CertificateDer> =
                    certs(cert_file).collect::<Result<_, _>>().map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cert")
                    })?;

                let key = private_key(key_file)
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key")
                    })?
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "no key")
                    })?;

                Ok(rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert_chain, key)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?)
            }
            TlsConfig::SelfSigned { hosts } => {
                let cert = generate_simple_self_signed(hosts)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                // Parse PEM directly from strings (no files!)
                let cert_pem = cert.cert.pem();
                let key_pem = cert.signing_key.serialize_pem();

                let cert_chain: Vec<CertificateDer> = certs(&mut Cursor::new(cert_pem.as_bytes()))
                    .collect::<Result<_, _>>()
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cert")
                    })?;

                let key = private_key(&mut Cursor::new(key_pem.as_bytes()))
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key")
                    })?
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "no key")
                    })?;

                Ok(rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert_chain, key)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?)
            }
        }
    }

    /// Starts a web listener with the provided config and services
    ///
    /// # Arguments
    /// * `cfg` - [`ServerConfig`] indicating listen options (ports, threads, TLS, etc)
    /// * `configure` - Closure accepting an Actix `ServiceConfig` which configures path handlers
    /// see <https://actix.rs/docs/application>
    ///
    /// # Behavior
    /// Enabling HTTP support automatically enabled H2C support, however requires explicit H2C
    /// connection from clients. Enabling HTTPS automatically supports HTTP2, with advertised
    /// upgrades for plain HTTPS clients if they support h2.
    ///
    /// Server spawns in the background. User responsible for shutdown hooks and
    /// calling [`stop()'] to shutdown the server gracefully.
    pub async fn listen<F>(cfg: ServerConfig, configure: F) -> Result<Server, std::io::Error>
    where
        F: Fn(&mut web::ServiceConfig) + Send + Sync + Clone + 'static,
    {
        let mut app = HttpServer::new(move || {
            App::new()
                .wrap(Compress::default())
                .configure(configure.clone())
        })
        .backlog(cfg.tcp_backlog.unwrap_or(4096))
        .max_connections(cfg.max_conns.unwrap_or(1 << 15))
        .workers(
            cfg.threads
                .unwrap_or(std::thread::available_parallelism()?.get()),
        )
        .client_request_timeout(Duration::from_secs(1))
        .max_connection_rate(cfg.tls_rate_per_worker.unwrap_or(256))
        .disable_signals();

        if let Some(http_port) = cfg.http_port {
            app = app.bind_auto_h2c((LISTEN_ADDR, http_port))?;
        }

        if let Some(tls) = cfg.tls {
            let server_cfg = Self::build_tls(tls)?;
            app = app.bind_rustls_0_23((LISTEN_ADDR, cfg.ssl_port.unwrap_or(443)), server_cfg)?
        }

        let run = app.run();
        let handle = run.handle();

        rt::spawn(async move { run.await });

        Ok(Self { handle })
    }

    /// Gracefully shutdown the web server
    pub async fn stop(&self) {
        self.handle.stop(true).await
    }
}
