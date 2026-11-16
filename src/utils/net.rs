use std::io;
use std::sync::{Arc, OnceLock};
use core::{task, time, ops};
use core::pin::Pin;
use core::future::Future;
pub use core::net::SocketAddr;

pub use tokio::net::TcpStream;
use tokio::io::{AsyncWrite, AsyncWriteExt, AsyncRead, AsyncReadExt};

pub fn rustls_config() -> &'static Arc<rustls::ClientConfig> {
    static CONFIG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();

    CONFIG.get_or_init(|| {
        let certs = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec()
        };

        Arc::new(rustls::ClientConfig::builder().with_root_certificates(certs).with_no_client_auth())
    })
}

pub struct TlsConnector<'a> {
    inner: tokio_rustls::TlsConnector,
    server_name: rustls_pki_types::ServerName<'a>,
}

impl<'a> TlsConnector<'a> {
    pub fn new(destination: &'a super::Url) -> Result<Self, rustls_pki_types::InvalidDnsNameError> {
        let server_name = rustls_pki_types::ServerName::try_from(destination.host())?;
        let inner = tokio_rustls::TlsConnector::from(rustls_config().clone());
        Ok(Self {
            inner,
            server_name,
        })
    }

    pub async fn connect(&self, tcp: TcpStream, timeout: time::Duration) -> Result<tokio_rustls::client::TlsStream<TcpStream>, io::Error> {
        let tls = self.inner.connect(self.server_name.to_owned(), tcp);
        if timeout.is_zero() {
            tls.await
        } else {
            match tokio::time::timeout(timeout, tls).await {
                Ok(result) => result,
                Err(error) => Err(io::Error::new(io::ErrorKind::TimedOut, error)),
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum TcpClient {
    Tcp(TcpStream),
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
}

impl AsyncRead for TcpClient {
    #[inline(always)]
    fn poll_read(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> task::Poll<io::Result<()>> {
        let this = self.get_mut();
        match this {
            Self::Tcp(socket) => AsyncRead::poll_read(Pin::new(socket), ctx, buf),
            Self::Tls(socket) => AsyncRead::poll_read(Pin::new(socket), ctx, buf),
        }
    }
}

impl AsyncWrite for TcpClient {
    #[inline(always)]
    fn poll_write(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, buf: &[u8]) -> task::Poll<io::Result<usize>> {
        let this = self.get_mut();
        match this {
            Self::Tcp(socket) => AsyncWrite::poll_write(Pin::new(socket), ctx, buf),
            Self::Tls(socket) => AsyncWrite::poll_write(Pin::new(socket), ctx, buf),
        }
    }

    #[inline(always)]
    fn poll_flush(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<io::Result<()>> {
        let this = self.get_mut();
        match this {
            Self::Tcp(socket) => AsyncWrite::poll_flush(Pin::new(socket), ctx),
            Self::Tls(socket) => AsyncWrite::poll_flush(Pin::new(socket), ctx),
        }
    }

    #[inline(always)]
    fn poll_shutdown(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<io::Result<()>> {
        let this = self.get_mut();
        match this {
            Self::Tcp(socket) => AsyncWrite::poll_shutdown(Pin::new(socket), ctx),
            Self::Tls(socket) => AsyncWrite::poll_shutdown(Pin::new(socket), ctx),
        }
    }

    #[inline(always)]
    fn is_write_vectored(&self) -> bool {
        match self {
            Self::Tcp(socket) => AsyncWrite::is_write_vectored(socket),
            Self::Tls(socket) => AsyncWrite::is_write_vectored(socket)
        }
    }

    #[inline(always)]
    fn poll_write_vectored(self: Pin<&mut Self>, ctx: &mut task::Context<'_>, bufs: &[io::IoSlice<'_>]) -> task::Poll<io::Result<usize>> {
        let this = self.get_mut();
        match this {
            Self::Tcp(socket) => AsyncWrite::poll_write_vectored(Pin::new(socket), ctx, bufs),
            Self::Tls(socket) => AsyncWrite::poll_write_vectored(Pin::new(socket), ctx, bufs),
        }
    }
}

impl TcpClient {
    pub fn plain(tcp: TcpStream) -> Self {
        Self::Tcp(tcp)
    }

    #[inline(always)]
    pub async fn tls(tcp: TcpStream, tls: TlsConnector<'_>, timeout: time::Duration) -> Result<Self, io::Error> {
        Ok(Self::Tls(tls.connect(tcp, timeout).await?))
    }

    #[inline(always)]
    pub fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = io::Result<()>> + Send + 'a {
        AsyncWriteExt::write_all(self, buf)
    }

    #[inline(always)]
    pub async fn shutdown(self) {
        let _ = match self {
            Self::Tls(mut socket) => socket.shutdown().await,
            Self::Tcp(mut socket) => socket.shutdown().await,
        };
    }

    #[inline(always)]
    pub fn buffered<const N: usize>(self, timeout: time::Duration) -> Buffered<N, Self> {
        Buffered::new(self, timeout)
    }
}

///Wrapper for AsyncWriteExt + AsyncReadExt to provide internal buffer to read data into
pub struct Buffered<const N: usize, T> {
    inner: T,
    buffer: [u8; N],
    timeout: time::Duration,
}

impl<const N: usize, T: AsyncWriteExt + AsyncReadExt + Unpin> Buffered<N, T> {
    #[inline]
    pub fn new(inner: T, timeout: time::Duration) -> Self {
        Self {
            inner,
            buffer: [0; N],
            timeout,
        }
    }

    #[inline]
    pub async fn read(&mut self) -> io::Result<&[u8]> {
        if self.timeout.is_zero() {
            self.inner.read(&mut self.buffer).await.map(|size| &self.buffer[..size])
        } else {
            match tokio::time::timeout(self.timeout, self.inner.read(&mut self.buffer)).await {
                Ok(result) => result.map(|size| &self.buffer[..size]),
                Err(elapsed) => Err(io::Error::new(io::ErrorKind::TimedOut, elapsed)),
            }
        }
    }

    #[inline]
    pub async fn read_str(&mut self) -> io::Result<&str> {
        self.read().await.and_then(|resp| core::str::from_utf8(resp).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))).map(str::trim)
    }
}

impl<const N: usize> Buffered<N, TcpClient> {
    #[inline(always)]
    pub fn shutdown(self) -> impl Future<Output = ()> + Send + 'static {
        self.inner.shutdown()
    }
}

impl<const N: usize, T> ops::Deref for Buffered<N, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const N: usize, T> ops::DerefMut for Buffered<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
