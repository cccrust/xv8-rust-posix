/// Simple POSIX-style option parser.
///
/// Supports short options (`-v`, `-o FILE`) and long options (`--verbose`, `--output FILE`).
/// Stops at `--`. Returns `None` when no more options remain.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opt {
    Short(char),
    Long(String),
}

#[derive(Debug)]
pub struct OptParser {
    args: Vec<String>,
    pos: usize,
    chars: std::vec::IntoIter<char>,
    current_opt: Option<char>,
}

impl OptParser {
    pub fn new(args: &[String]) -> Self {
        let args = args.to_vec();
        OptParser {
            args,
            pos: 1, // skip program name
            chars: Vec::new().into_iter(),
            current_opt: None,
        }
    }

    /// Return the next option and its optional value.
    /// `optstr` lists short options that take a value (e.g. "o:" means `-o VAL`).
    /// `longopts` maps long option names to (short_char, takes_value).
    /// Returns `None` when no more options (or `--` encountered).
    /// Remaining non-option arguments can be retrieved via `remaining()`.
    pub fn next(&mut self, optstr: &str, longopts: &[(&str, Option<char>, bool)]) -> Option<(Opt, Option<String>)> {
        loop {
            // If we're in the middle of a short option group (e.g., -abc)
            if let Some(c) = self.current_opt {
                self.current_opt = None;
                let takes_val = optstr.contains(&format!("{}:", c));
                if takes_val {
                    // Value must be the next argument
                    let val = self.args.get(self.pos).cloned();
                    self.pos += 1;
                    return Some((Opt::Short(c), val));
                }
                // Check if next char in group exists
                if let Some(next_c) = self.chars.next() {
                    self.current_opt = Some(next_c);
                }
                return Some((Opt::Short(c), None));
            }

            let arg = self.args.get(self.pos)?;

            if arg == "--" {
                self.pos += 1;
                return None;
            }

            if arg.starts_with("--") {
                let (name, inline_val) = if let Some(eq_idx) = arg.find('=') {
                    (arg[2..eq_idx].to_string(), Some(arg[eq_idx+1..].to_string()))
                } else {
                    (arg[2..].to_string(), None)
                };

                self.pos += 1;

                let takes_val = longopts.iter().any(|(n, _, tv)| *n == name && *tv);
                let val = if inline_val.is_none() && takes_val {
                    self.args.get(self.pos).cloned().map(|v| { self.pos += 1; v })
                } else {
                    inline_val
                };
                return Some((Opt::Long(name), val));
            }

            if arg.starts_with('-') && arg.len() > 1 {
                let c = arg.as_bytes()[1] as char;
                let takes_val = optstr.contains(&format!("{}:", c));

                self.pos += 1;

                if takes_val {
                    let val = self.args.get(self.pos).cloned();
                    self.pos += 1;
                    return Some((Opt::Short(c), val));
                }

                // Check for grouped short options
                if arg.len() > 2 {
                    self.chars = arg[2..].chars().collect::<Vec<_>>().into_iter();
                    self.current_opt = self.chars.next();
                }

                return Some((Opt::Short(c), None));
            }

            // Not an option — stop
            break;
        }

        None
    }

    /// Return remaining (non-option) arguments.
    pub fn remaining(&self) -> &[String] {
        &self.args[self.pos..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_short() {
        let args = vec!["prog".into(), "-v".into(), "file".into()];
        let mut p = OptParser::new(&args);
        assert_eq!(p.next("", &[]), Some((Opt::Short('v'), None)));
        assert_eq!(p.remaining(), &["file"]);
    }

    #[test]
    fn test_short_with_value() {
        let args = vec!["prog".into(), "-o".into(), "out.txt".into()];
        let mut p = OptParser::new(&args);
        assert_eq!(p.next("o:", &[]), Some((Opt::Short('o'), Some("out.txt".into()))));
    }

    #[test]
    fn test_long() {
        let args = vec!["prog".into(), "--verbose".into(), "--output=out.txt".into()];
        let mut p = OptParser::new(&args);
        let longopts = &[("verbose", Some('v'), false), ("output", Some('o'), true)];
        assert_eq!(p.next("", longopts), Some((Opt::Long("verbose".into()), None)));
        assert_eq!(p.next("", longopts), Some((Opt::Long("output".into()), Some("out.txt".into()))));
    }

    #[test]
    fn test_double_dash_stops() {
        let args = vec!["prog".into(), "--".into(), "-v".into(), "file".into()];
        let mut p = OptParser::new(&args);
        assert_eq!(p.next("", &[]), None);
        assert_eq!(p.remaining(), &["-v", "file"]);
    }

    #[test]
    fn test_grouped_short() {
        let args = vec!["prog".into(), "-abc".into()];
        let mut p = OptParser::new(&args);
        assert_eq!(p.next("", &[]), Some((Opt::Short('a'), None)));
        assert_eq!(p.next("", &[]), Some((Opt::Short('b'), None)));
        assert_eq!(p.next("", &[]), Some((Opt::Short('c'), None)));
    }
}
