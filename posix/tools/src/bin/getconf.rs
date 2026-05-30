fn main() {
    let sys = get_config();
    for (key, val) in sys {
        println!("{}: {}", key, val);
    }
}

fn get_config() -> Vec<(&'static str, String)> {
    let mut v = Vec::new();
    v.push(("ARG_MAX", "4096".to_string()));
    v.push(("BC_BASE_MAX", "99".to_string()));
    v.push(("BC_DIM_MAX", "2048".to_string()));
    v.push(("BC_SCALE_MAX", "99".to_string()));
    v.push(("BC_STRING_MAX", "1000".to_string()));
    v.push(("CHARCLASS_NAME_MAX", "14".to_string()));
    v.push(("COLL_WEIGHTS_MAX", "2".to_string()));
    v.push(("EXPR_NEST_MAX", "32".to_string()));
    v.push(("LINE_MAX", "2048".to_string()));
    v.push(("NGROUPS_MAX", "16".to_string()));
    v.push(("OPEN_MAX", "256".to_string()));
    v.push(("PASS_MAX", "8".to_string()));
    v.push(("PATH_MAX", "256".to_string()));
    v.push(("PIPE_BUF", "512".to_string()));
    v.push(("RE_DUP_MAX", "255".to_string()));

    #[cfg(unix)]
    unsafe {
        use libc::sysconf;
        v.push(("PAGESIZE", sysconf(libc::_SC_PAGESIZE).to_string()));
        v.push(("CLK_TCK", sysconf(libc::_SC_CLK_TCK).to_string()));
        v.push(("NPROCESSORS_CONF", sysconf(libc::_SC_NPROCESSORS_CONF).to_string()));
        v.push(("NPROCESSORS_ONLN", sysconf(libc::_SC_NPROCESSORS_ONLN).to_string()));
        v.push(("PHYS_PAGES", sysconf(libc::_SC_PHYS_PAGES).to_string()));
    }
    v
}
