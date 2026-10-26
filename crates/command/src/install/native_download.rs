use crate::cache::download::{is_complete, publish};
use anyhow::{Context, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_DOWNLOADS: usize = 4;

pub(super) fn download_files(urls: &[String], targets: &[String]) -> anyhow::Result<()> {
    if urls.len() != targets.len() {
        bail!("Download URL and cache path counts do not match");
    }
    // Cache aliases may point at the same file; write each destination only once.
    let mut seen = std::collections::HashSet::new();
    let pending: Vec<_> = urls
        .iter()
        .zip(targets)
        .filter(|(_, target)| !is_complete(Path::new(target)) && seen.insert(target.as_str()))
        .collect();
    if pending.is_empty() {
        return Ok(());
    }
    let progress = MultiProgress::new();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(pending.len().min(MAX_DOWNLOADS))
        .build()?;
    pool.install(|| {
        // Blocking reqwest must initialize and drop outside Tokio workers.
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(600))
            .build()
            .context("Failed to build HTTP client")?;
        pending
            .par_iter()
            .try_for_each(|(url, target)| download_file(&client, url, Path::new(target), &progress))
    })
}

// Never expose a partially downloaded file as a valid cache hit on the next invocation.
struct PendingFile(PathBuf);

impl Drop for PendingFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn download_file(
    client: &Client,
    url: &str,
    target: &Path,
    progress: &MultiProgress,
) -> anyhow::Result<()> {
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to download from {url}"))?
        .error_for_status()?;
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let (pending, mut file) = create_pending_file(target)?;
    let length = response.content_length();
    let bar = progress.add(ProgressBar::new(length.unwrap_or(0)));
    bar.set_message(
        target
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
    );
    bar.set_style(ProgressStyle::with_template(
        "{msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
    )?);
    let result = (|| -> anyhow::Result<()> {
        let mut buffer = [0u8; 64 * 1024];
        let mut downloaded = 0u64;
        loop {
            let len = response.read(&mut buffer)?;
            if len == 0 {
                break;
            }
            file.write_all(&buffer[..len])?;
            downloaded += len as u64;
            bar.set_position(downloaded);
        }
        if length.is_some_and(|expected| expected != downloaded) {
            bail!("Incomplete download from {url}");
        }
        file.flush()?;
        Ok(())
    })();
    // Close before rename or cleanup, including on Windows.
    drop(file);
    if let Err(error) = result {
        bar.abandon_with_message("download failed");
        return Err(error);
    }
    publish(&pending.0, target)?;
    bar.finish_with_message("downloaded");
    Ok(())
}

fn create_pending_file(target: &Path) -> anyhow::Result<(PendingFile, File)> {
    loop {
        let mut name = target.as_os_str().to_os_string();
        name.push(format!(".{}.part", rand::random::<u64>()));
        let path = PathBuf::from(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((PendingFile(path), file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::net::TcpListener;

    #[tokio::test(flavor = "multi_thread")]
    async fn native_download_is_safe_when_called_from_tokio() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("file").to_string_lossy().into_owned();
        assert!(download_files(&["not-a-url".into()], &[target]).is_err());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn native_download_bounds_parallel_requests_and_keeps_url_target_pairs() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let temp = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let max_active = peak.clone();
        let server = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut handlers = Vec::new();
            while handlers.len() < 6 && std::time::Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut socket, _)) => {
                        let active = active.clone();
                        let peak = peak.clone();
                        handlers.push(std::thread::spawn(move || {
                            socket.set_nonblocking(false).unwrap();
                            socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                            let mut request = String::new();
                            std::io::BufRead::read_line(
                                &mut std::io::BufReader::new(&mut socket),
                                &mut request,
                            ).unwrap();
                            let body = request.split_whitespace().nth(1).unwrap();
                            let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                            peak.fetch_max(count, Ordering::SeqCst);
                            std::thread::sleep(Duration::from_millis(100));
                            active.fetch_sub(1, Ordering::SeqCst);
                            write!(socket, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
                        }));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1))
                    }
                    Err(error) => panic!("{error}"),
                }
            }
            assert_eq!(handlers.len(), 6);
            for handler in handlers {
                handler.join().unwrap();
            }
        });
        let urls: Vec<_> = (0..6).map(|i| format!("{base_url}/{i}")).collect();
        let targets: Vec<_> = (0..6)
            .map(|i| {
                temp.path()
                    .join(i.to_string())
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        download_files(&urls, &targets).unwrap();
        server.join().unwrap();
        assert!((2..=MAX_DOWNLOADS).contains(&max_active.load(Ordering::SeqCst)));
        for (i, target) in targets.iter().enumerate() {
            assert_eq!(std::fs::read_to_string(target).unwrap(), format!("/{i}"));
        }
    }

    #[test]
    fn native_download_publishes_complete_files_and_preserves_cache() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp
            .path()
            .join("archive.zip")
            .to_string_lossy()
            .into_owned();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/file", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            socket.read(&mut request).unwrap();
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc")
                .unwrap();
        });
        download_files(&[url.clone(), url], &[target.clone(), target.clone()]).unwrap();
        server.join().unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"abc");
        download_files(&["invalid-url".into()], std::slice::from_ref(&target)).unwrap();
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn cache_regression_native_fallback_replaces_aria2_partial_and_preserves_other_hits() {
        use crate::cache::download::with_suffix;
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("partial.zip");
        let cached = temp.path().join("cached.zip");
        std::fs::write(&target, b"preallocated partial").unwrap();
        std::fs::write(with_suffix(&target, ".aria2"), b"resume").unwrap();
        std::fs::write(with_suffix(&target, ".incomplete"), b"partial").unwrap();
        std::fs::write(with_suffix(&target, ".incomplete.aria2"), b"resume").unwrap();
        std::fs::write(&cached, b"keep").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/file", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = [0; 1024];
            socket.read(&mut request).unwrap();
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc")
                .unwrap();
        });
        download_files(
            &["invalid-url".into(), url],
            &[
                cached.to_string_lossy().into_owned(),
                target.to_string_lossy().into_owned(),
            ],
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"abc");
        assert_eq!(std::fs::read(&cached).unwrap(), b"keep");
        assert!(is_complete(&target));
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn native_download_removes_partial_files_on_disconnect() {
        let temp = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/file", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            socket.read(&mut request).unwrap();
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nabc",
                )
                .unwrap();
        });
        let target = temp
            .path()
            .join("archive.zip")
            .to_string_lossy()
            .into_owned();
        assert!(download_files(&[url], &[target]).is_err());
        server.join().unwrap();
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }
}
