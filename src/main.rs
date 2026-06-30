fn main() {
    if let Err(e) = terminal_guru::run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
