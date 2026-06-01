# Memory Log for xv8-rust-posix

## What has been done

1. **shell.sh**: Created to build and launch POSIX shell with network tools.
   - Builds posix/tools and net/tools (release)
   - Adds their release binaries to PATH
   - Executes the POSIX shell (sh) with prompt "posix> "

2. **Modified sh.rs**: Changed the REPL prompt from "$ " to "posix> ".

3. **Added network tools** (in net/tools/src/bin/):
   - netstat.rs: Simplified netstat (Linux only)
   - http_server.rs: Tiny HTTP server serving current directory on port 8080
   - curl.rs: Simple HTTP GET client using ureq
   - ssh_client.rs: Placeholder SSH client (prints arguments)
   - ssh_server.rs: Placeholder SSH server (not yet implemented due to complexity)

4. **Updated net/tools/Cargo.toml**:
   - Added bin declarations for the new tools
   - Added dependencies: tiny_http, ureq, ssh2, tokio, hyper, rand

5. **Created _doc/plan.md**: Outlines planned network tools to add.

## What needs to be done next

- Implement real ssh_client.rs and ssh_server.rs (may require elevated privileges or be complex).
- Add more network tools from plan.md (traceroute, nc, ftp, etc.).
- Test each tool to ensure they work as expected.
- Consider adding a tool to generate RSA keys for SSH.
- Ensure the shell.sh script works after each addition (rebuilds automatically).

## How to test

Run:
```bash
./shell.sh
```
Then inside the shell, try commands like:
- `ls` (should show posix tools)
- `ping 8.8.8.8`
- `host google.com`
- `curl http://example.com`
- `netstat -tuln` (if on Linux)
- In another terminal, start the http_server: `http_server` and curl it.

## Notes

- The shell prompt is now "posix> " to distinguish from host shell.
- All tools are built for the host machine (no QEMU/xv8 needed).
- Some tools (like netstat) are platform-specific (Linux only in current implementation).
