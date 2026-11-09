//! Native terminal entry point for the shared command engine.
fn main() {
    #[cfg(feature = "net-diagnostics")]
    if std::env::var_os("KHOR_NET_DIAGNOSTICS").is_some() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("khor_net_samples=debug")
            .with_target(true)
            .with_ansi(false)
            .with_writer(std::io::stderr)
            .try_init();
    }
    if let Some(code) = khor_commands::console_main_if_requested(start_web) {
        std::process::exit(code);
    }
    let code = khor_commands::run_cli(start_web, write);
    std::process::exit(code);
}

fn start_web(root: &std::path::Path) {
    match khor_web::listen(root.to_path_buf(), khor_web::port()) {
        Ok(face) => eprintln!("{}", khor_catalog::cli::web_banner(face.addr)),
        Err(error) => eprintln!("{error}"),
    }
}

/// Native pipe handling belongs to the process entry point, never embedded requests.
fn write(stderr: bool, text: &str) {
    use std::io::Write;
    if stderr {
        let _ = std::io::stderr().lock().write_all(text.as_bytes());
        return;
    }
    if let Err(error) = std::io::stdout().lock().write_all(text.as_bytes()) {
        if error.kind() == std::io::ErrorKind::BrokenPipe {
            #[cfg(unix)]
            unsafe {
                libc::signal(libc::SIGPIPE, libc::SIG_DFL);
                libc::raise(libc::SIGPIPE);
            }
            std::process::exit(141);
        }
        panic!("failed printing to stdout: {error}");
    }
}
