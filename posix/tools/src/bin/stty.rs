fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        if arg.starts_with('-') { continue; }
    }
    // stub: prints current terminal settings (minimal)
    println!("speed 38400 baud; line = 0;");
    println!("intr = ^C; quit = ^\\; erase = ^H; kill = ^U;");
}
