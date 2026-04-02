use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

/// Locate the bundled server binary.
///
/// - In a production Tauri bundle the server sits next to the app binary.
/// - In development we fall back to the cargo debug/release build output.
fn find_server_binary() -> Option<PathBuf> {
    // 1. Next to the running Tauri executable (production bundle)
    if let Ok(exe) = std::env::current_exe() {
        for name in &["server", "server.exe"] {
            let candidate = exe.parent()?.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. Cargo debug build (dev mode)
    let workspace = std::env::current_dir().ok()?;
    for profile in &["debug", "release"] {
        for name in &["server", "server.exe"] {
            let candidate = workspace.join("server/target").join(profile).join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Poll port 8080 until the server is accepting connections or we time out.
fn wait_for_server(timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if TcpStream::connect_timeout(
            &"127.0.0.1:8080".parse().unwrap(),
            Duration::from_secs(1),
        )
        .is_ok()
        {
            return true;
        }
        thread::sleep(Duration::from_millis(250));
    }
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let server: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let server_for_cleanup = Arc::clone(&server);

    // Attempt to launch the backend server subprocess.
    // If it is already running (e.g. the user started it manually), we skip this.
    if TcpStream::connect("127.0.0.1:8080").is_err() {
        if let Some(bin) = find_server_binary() {
            eprintln!("[IPA Tool] Starting backend server: {}", bin.display());
            match Command::new(&bin).env("RUST_LOG", "info").spawn() {
                Ok(child) => {
                    *server.lock().unwrap() = Some(child);
                    if wait_for_server(30) {
                        eprintln!("[IPA Tool] Backend server is ready.");
                    } else {
                        eprintln!("[IPA Tool] Warning: server did not become ready in 30 s.");
                    }
                }
                Err(e) => {
                    eprintln!("[IPA Tool] Failed to start backend server: {}", e);
                }
            }
        } else {
            eprintln!(
                "[IPA Tool] Server binary not found. \
                 Please run `pnpm build:rust` first, or start the server manually with `pnpm dev:rust`."
            );
        }
    } else {
        eprintln!("[IPA Tool] Backend server already running on port 8080.");
    }

    tauri::Builder::default()
        .on_window_event(move |_window, event| {
            // Kill the server subprocess when the last window is closed.
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(mut child) = server_for_cleanup.lock().unwrap().take() {
                    let _ = child.kill();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
