use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: dirname string");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    // POSIX: dirname / -> /
    //         dirname // -> // (may be implementation-defined)
    //         dirname /foo -> /
    //         dirname foo -> .
    //         dirname foo/bar -> foo
    let parent = if args[1] == "/" || args[1] == "//" {
        args[1].clone()
    } else {
        match path.parent() {
            Some(p) if p.as_os_str().is_empty() => ".".to_string(),
            Some(p) => p.to_string_lossy().to_string(),
            None => ".".to_string(),
        }
    };

    println!("{}", parent);
}
