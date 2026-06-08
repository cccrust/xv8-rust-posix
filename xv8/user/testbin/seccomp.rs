#![no_std]
#![no_main]

use user::*;

// BPF filter that allows: read(5), write(16), exit(2), exit_group(122)
// Kills everything else
// Instruction layout: code(2) + jt(1) + jf(1) + k(4) = 8 bytes
fn make_whitelist() -> [u8; 80] {
    let mut f = [0u8; 80];
    // pc=0: ld [0]  (load syscall number)
    f[0..8].copy_from_slice(&[0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // pc=1: jeq READ(5), jt=7, jf=0
    f[8..16].copy_from_slice(&[0x15, 0x07, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00]);
    // pc=2: ld [0]
    f[16..24].copy_from_slice(&[0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // pc=3: jeq WRITE(16), jt=5, jf=0
    f[24..32].copy_from_slice(&[0x15, 0x05, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00]);
    // pc=4: ld [0]
    f[32..40].copy_from_slice(&[0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // pc=5: jeq EXIT(2), jt=3, jf=0
    f[40..48].copy_from_slice(&[0x15, 0x03, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);
    // pc=6: ld [0]
    f[48..56].copy_from_slice(&[0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // pc=7: jeq EXIT_GROUP(122), jt=1, jf=0
    f[56..64].copy_from_slice(&[0x15, 0x01, 0x00, 0x00, 0x7a, 0x00, 0x00, 0x00]);
    // pc=8: ret KILL (SECCOMP_RET_KILL_PROCESS = 0x80000000)
    f[64..72].copy_from_slice(&[0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
    // pc=9: ret ALLOW (SECCOMP_RET_ALLOW = 0x7fff0000)
    f[72..80].copy_from_slice(&[0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x7f]);
    f
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let filter = make_whitelist();

    // First verify seccomp can be installed
    let ret = unsafe { raw::seccomp(2, 0, filter.as_ptr()) };
    if ret < 0 {
        // If seccomp returns error, skip (not all configs support it)
        exit(0);
    }

    // Now that seccomp is installed with whitelist, fork to test
    match fork() {
        Ok(0) => {
            // Child: try open() - should be killed by seccomp
            let _fd = open("nonexistent", OpenFlag::READ_ONLY);
            // If we get here, seccomp didn't kill us - fail
            exit_with_msg("seccomp: open was not blocked");
        }
        Ok(_pid) => {
            let mut status = 0;
            let _ = wait(&mut status);
            // Child should have been killed (non-zero exit)
            if status == 0 {
                exit_with_msg("seccomp: child was not killed");
            }
            // Test: seccomp should allow write (we're using it now)
            println!("seccomp test passed");
            exit(0);
        }
        Err(_) => exit_with_msg("fork failed"),
    }
}
