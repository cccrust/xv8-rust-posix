use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut bs: u64 = 512;
    let mut count: Option<u64> = None;
    let mut seek_out: u64 = 0;
    let mut skip_in: u64 = 0;
    let mut conv = String::new();
    let mut ifile: Option<String> = None;
    let mut ofile: Option<String> = None;

    while i < args.len() {
        if args[i] == "if" && i + 1 < args.len() {
            ifile = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "of" && i + 1 < args.len() {
            ofile = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "bs" && i + 1 < args.len() {
            bs = args[i + 1].parse().unwrap_or(512);
            i += 2;
        } else if args[i] == "count" && i + 1 < args.len() {
            count = Some(args[i + 1].parse().unwrap_or(0));
            i += 2;
        } else if args[i] == "seek" && i + 1 < args.len() {
            seek_out = args[i + 1].parse().unwrap_or(0) * bs;
            i += 2;
        } else if args[i] == "skip" && i + 1 < args.len() {
            skip_in = args[i + 1].parse().unwrap_or(0) * bs;
            i += 2;
        } else if args[i] == "conv" && i + 1 < args.len() {
            conv = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut input: Box<dyn Read> = match ifile {
        Some(ref path) => Box::new(File::open(path).unwrap_or_else(|e| {
            eprintln!("dd: {}: {}", path, e);
            std::process::exit(1);
        })),
        None => Box::new(io::stdin().lock()),
    };
    let mut output: Box<dyn Write> = match ofile {
        Some(ref path) => Box::new(File::create(path).unwrap_or_else(|e| {
            eprintln!("dd: {}: {}", path, e);
            std::process::exit(1);
        })),
        None => Box::new(io::stdout().lock()),
    };

    if skip_in > 0 {
        let mut buf = [0u8; 4096];
        let mut skipped = 0u64;
        while skipped < skip_in {
            let n = input.read(&mut buf[..(skip_in - skipped).min(4096) as usize]).unwrap_or(0);
            if n == 0 { break; }
            skipped += n as u64;
        }
    }
    if seek_out > 0 {
        if let Some(ref path) = ofile {
            let mut f = File::open(path).unwrap_or_else(|e| {
                eprintln!("dd: {}: {}", path, e);
                std::process::exit(1);
            });
            f.seek(SeekFrom::Start(seek_out)).ok();
            drop(f);
        }
    }

    let lcase = conv == "lcase";
    let ucase = conv == "ucase";
    let mut buf = vec![0u8; bs as usize];
    let mut total = 0u64;
    let max_blocks = count.unwrap_or(u64::MAX);
    let mut blocks = 0u64;

    loop {
        let n = input.read(&mut buf).unwrap_or(0);
        if n == 0 || blocks >= max_blocks { break; }
        let mut data = buf[..n].to_vec();
        if lcase { data.make_ascii_lowercase(); }
        if ucase { data.make_ascii_uppercase(); }
        output.write_all(&data).ok();
        total += n as u64;
        blocks += 1;
    }

    eprintln!("{} bytes copied", total);
}
