fn main() {
    let users = get_users();
    println!("{}", users.join(" "));
}

fn get_users() -> Vec<String> {
    #[cfg(unix)]
    {
        let mut seen = std::collections::BTreeSet::new();
        unsafe {
            libc::setutxent();
            loop {
                let ptr = libc::getutxent();
                if ptr.is_null() { break; }
                let ut = &*ptr;
                if ut.ut_type == libc::USER_PROCESS {
                    let user = std::ffi::CStr::from_ptr(ut.ut_user.as_ptr())
                        .to_string_lossy().to_string();
                    if !user.is_empty() {
                        seen.insert(user);
                    }
                }
            }
            libc::endutxent();
        }
        seen.into_iter().collect()
    }
    #[cfg(not(unix))]
    {
        vec!["root".to_string()]
    }
}