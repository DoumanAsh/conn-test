use core::future::Future;
use core::pin::Pin;
use core::task;
use std::io;

///Future that resolves when program is requested to terminate
#[must_use = "Future does nothing unless polled"]
pub struct Termination {
    #[cfg(unix)]
    term: tokio::signal::unix::Signal,
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(windows)]
    interrupt: tokio::signal::windows::CtrlC,
}

impl Termination {
    #[inline]
    ///Creates termination future, returning error if underlying signal handler cannot be created
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            #[cfg(unix)]
            term: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?,
            #[cfg(unix)]
            interrupt: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?,
            #[cfg(windows)]
            interrupt: tokio::signal::windows::ctrl_c()?,
        })
    }
}

impl Future for Termination {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let this = self.get_mut();

        #[cfg(unix)]
        {
            if let task::Poll::Ready(Some(())) = this.term.poll_recv(ctx) {
                return task::Poll::Ready(());
            }
        }
        if let task::Poll::Ready(Some(())) = this.interrupt.poll_recv(ctx) {
            return task::Poll::Ready(());
        }

        task::Poll::Pending
    }
}
