use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};

pub struct StderrCapture {
    buffer: Arc<Mutex<Vec<u8>>>,
    saved_fd: OwnedFd,
}

impl StderrCapture {
    pub fn install() -> Result<Self> {
        let saved_fd = unsafe {
            let raw = libc::dup(libc::STDERR_FILENO);
            if raw < 0 {
                anyhow::bail!("dup(stderr) failed: {}", std::io::Error::last_os_error());
            }
            OwnedFd::from_raw_fd(raw)
        };

        let mut fds: [libc::c_int; 2] = [0; 2];
        let rc = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if rc < 0 {
            anyhow::bail!("pipe() failed: {}", std::io::Error::last_os_error());
        }
        let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };

        let rc = unsafe { libc::dup2(write_fd.as_raw_fd(), libc::STDERR_FILENO) };
        if rc < 0 {
            anyhow::bail!("dup2(pipe -> stderr) failed: {}", std::io::Error::last_os_error());
        }
        drop(write_fd);

        let buffer = Arc::new(Mutex::new(Vec::<u8>::with_capacity(4096)));
        let reader_buffer = buffer.clone();

        thread::Builder::new()
            .name("tui-stderr-capture".into())
            .spawn(move || {
                let mut file = unsafe { std::fs::File::from_raw_fd(read_fd.into_raw_fd()) };
                let mut chunk = [0u8; 1024];
                loop {
                    match file.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut buf) = reader_buffer.lock() {
                                if buf.len() < 64 * 1024 {
                                    buf.extend_from_slice(&chunk[..n]);
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .context("Failed to spawn stderr capture thread")?;

        Ok(Self { buffer, saved_fd })
    }

    pub fn take(&self) -> Option<String> {
        let mut guard = self.buffer.lock().ok()?;
        if guard.is_empty() {
            return None;
        }
        let bytes = std::mem::take(&mut *guard);
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn peek_nonempty(&self) -> bool {
        self.buffer.lock().map(|g| !g.is_empty()).unwrap_or(false)
    }
}

impl Drop for StderrCapture {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.saved_fd.as_raw_fd(), libc::STDERR_FILENO);
        }
    }
}
