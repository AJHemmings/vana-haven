use std::fs;
use std::net::TcpListener;
use std::path::Path;

pub const DEFAULT_PORT: u16 = 24244;

/// Binds to `DEFAULT_PORT`, falling back to an OS-assigned free port if that
/// port is already in use. When a fallback is used, the actual port is written
/// to `port_file_path` so the addon can discover it after trying the default.
pub fn bind_with_fallback(port_file_path: &Path) -> std::io::Result<TcpListener> {
    match TcpListener::bind(("127.0.0.1", DEFAULT_PORT)) {
        Ok(listener) => {
            let _ = fs::remove_file(port_file_path);
            Ok(listener)
        }
        Err(_) => {
            let listener = TcpListener::bind(("127.0.0.1", 0))?;
            let actual_port = listener.local_addr()?.port();
            fs::write(port_file_path, actual_port.to_string())?;
            Ok(listener)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener as StdTcpListener;

    #[test]
    fn falls_back_and_writes_port_file_when_default_port_busy() {
        let _blocker = StdTcpListener::bind(("127.0.0.1", DEFAULT_PORT))
            .expect("default port must be free to run this test");

        let tmp_dir = std::env::temp_dir().join(format!("vana-haven-test-{}", std::process::id()));
        fs::create_dir_all(&tmp_dir).unwrap();
        let port_file = tmp_dir.join("port.txt");

        let listener = bind_with_fallback(&port_file).unwrap();
        let bound_port = listener.local_addr().unwrap().port();

        assert_ne!(bound_port, DEFAULT_PORT);
        assert_eq!(fs::read_to_string(&port_file).unwrap(), bound_port.to_string());

        fs::remove_dir_all(&tmp_dir).unwrap();
    }
}
