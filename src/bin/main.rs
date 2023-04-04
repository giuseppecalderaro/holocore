fn main() {
    let args: std::env::Args = std::env::args();
    let argv: Vec<String> = args.collect();
    let argc: usize = argv.len();

    match holocore::entry_point(argc, argv) {
        Ok(()) => (),
        Err(e) => panic!("Guru Meditation: {}", e)
    }
}
