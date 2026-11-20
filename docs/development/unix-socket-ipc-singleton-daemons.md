---
title: "Unix-Socket IPC and Singleton Local Daemons: Lifetime Locks, Stale Sockets, Peer Credentials, Socket Activation"
category: concurrency
languages: [rust, python, shell]
complexity: advanced
use_cases:
  - "making a CLI-launched local daemon (broker, agent, language server) a true singleton without pidfile races"
  - "handling a leftover socket file safely on macOS and Linux, and knowing when a refused connect does not mean stale"
  - "choosing where a Unix socket lives, what mode it and its directory need, and which peer-credential call to trust"
  - "framing a request/response protocol over a stream socket (JSONL, bounded lines, SIGPIPE) and passing file descriptors"
  - "deciding between a self-daemonizing process and systemd or launchd socket activation"
summary: "Primary-source and experiment-backed guidance for local daemons that serve a CLI over a Unix domain socket: an exclusive flock held for the daemon's life is the singleton and the socket is derived state; bind fails on any existing path and the kernel never removes it; connect-refused means stale only when you hold the lock (a full backlog also refuses on macOS); the 0700 directory is the portable access control while socket modes are enforced on Linux and macOS but not by POSIX; peer uid/gid from SO_PEERCRED, getpeereid or LOCAL_PEERCRED is reliable and the pid is not; JSONL with a hard line limit; SCM_RIGHTS rules; systemd and launchd socket activation with worked unit files; two CVEs from world-writable sockets."
provenance: researched
researched: 2026-11-02
sources:
  - https://www.man7.org/linux/man-pages/man7/unix.7.html
  - https://man7.org/linux/man-pages/man2/flock.2.html
  - https://www.mankier.com/2/fcntl
  - https://man7.org/linux/man-pages/man2/connect.2.html
  - https://man7.org/linux/man-pages/man3/cmsg.3.html
  - https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html
  - https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html
  - https://man.freebsd.org/cgi/man.cgi?query=unix&sektion=4
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/un.h
  - https://developer.apple.com/forums/thread/850777
  - https://www.freedesktop.org/software/systemd/man/latest/systemd.socket.html
  - https://www.freedesktop.org/software/systemd/man/latest/sd_listen_fds.html
  - https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html
  - https://man.cx/daemon(7)
  - https://keith.github.io/xcode-man-pages/launchd.plist.5.html
  - https://developer.apple.com/documentation/xpc/launch_activate_socket
  - https://lists.gnupg.org/pipermail/gnupg-devel/2018-June/033789.html
  - https://doc.rust-lang.org/stable/src/std/os/unix/net/ucred.rs.html
  - https://doc.rust-lang.org/std/os/linux/net/trait.SocketAddrExt.html
  - https://doc.rust-lang.org/stable/src/std/sys/pal/unix/mod.rs.html
  - https://doc.rust-lang.org/unstable-book/language-features/unix-sigpipe.html
  - https://github.com/rust-lang/rust/pull/101077/files
  - https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html
  - https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html
  - https://docs.rs/tokio/latest/tokio/net/unix/struct.UCred.html
  - https://docs.rs/tokio-util/latest/tokio_util/codec/struct.LinesCodec.html
  - https://docs.rs/fd-lock/latest/fd_lock/
  - https://docs.rs/crate/moq-native/latest/source/src/unix.rs
  - https://crosvm.dev/doc/base/sys/linux/struct.UnlinkUnixListener.html
  - https://docs.rs/crate/brokk-mj-core/2.7.2/source/src/local_sockets.rs
  - https://qemu.googlesource.com/qemu/+/f9922937d173f50fe59fd1b20fadc445fb6b2564
  - https://github.com/howjmay/go/blob/c6109c495ca74cd22c7ab7c1af6e574962f79d01/src/net/unixsock_posix.go
  - https://go.dev/doc/go1.8
  - https://github.com/caddyserver/caddy/blob/master/listen_unix.go
  - https://docs.python.org/3/library/asyncio-stream.html
  - https://docs.python.org/3/library/asyncio-eventloop.html
  - https://nodejs.org/api/net.html
  - https://gist.github.com/visnup/9801864
  - https://jsonlines.org/
  - https://docs.docker.com/engine/install/linux-postinstall/
  - https://docs.docker.com/engine/security/
  - https://docs.docker.com/reference/cli/dockerd/
  - https://book.hacktricks.xyz/linux-hardening/privilege-escalation/docker-security/docker-breakout-privilege-escalation
  - https://dbus.freedesktop.org/doc/dbus-specification.html
  - https://gitlab.freedesktop.org/dbus/dbus/blob/master/dbus/dbus-auth.c
  - https://gitlab.freedesktop.org/dbus/dbus/-/merge_requests/398
  - https://sources.debian.org/src/dbus/1.16.2-4/NEWS
  - https://lpc.events/event/18/contributions/1691/attachments/1499/3166/-LPC-2024-%20PID%20FDs-%20where%20we%20were,%20where%20we%20are%20and%20were%20we%20would%20like%20to%20go.pdf
  - https://lwn.net/Articles/1018746/
  - https://postgresqlco.nf/doc/en/param/unix_socket_permissions/
  - https://manpages.debian.org/unstable/openssh-client-gssapi/ssh-agent.1.en.html
  - https://man.openbsd.org/tmux.1
  - https://specifications.freedesktop.org/basedir/latest/
  - https://docs.podman.io/en/latest/markdown/podman-system-service.1.html
  - https://github.com/podman-container-tools/podman/blob/main/docs/tutorials/socket_activation.md
  - https://ubuntu.com/security/CVE-2015-8222
  - https://securelayer7.net/lab/cve-2026-53657-lima-qemu-guest-agent-socket-privilege-escalation
  - https://www.securityscientist.net/blog/12-questions-and-answers-about-unsafe-symlink-handling/
  - https://lettieri.iet.unipi.it/hacking/symlinks.pdf
  - https://www.tinc-vpn.org/git/?a=commitdiff&h=a38e0d621397d6d69c939ccc287d5a803b668195&p=tinc
  - https://gitlab.torproject.org/tpo/core/tor/-/issues/14186
  - https://github.com/php/php-src/pull/10973
  - https://docs.rs/crate/questdb-rs/7.0.0/source/src/ws/nosigpipe.rs
  - https://github.com/cppalliance/corosio/issues/230
  - https://git.tldp.org/LDP/man-pages/commits/commit/bf9cc31523d7c9b3c7d0a15e1e4e9c105a7ff7ac
  - https://sourceware.org/ml/glibc-cvs/2017-q2/msg00090.html
  - https://kernelnewbies.org/Linux_6.5
  - https://services.nvd.nist.gov/rest/json/cves/2.0?cveId=CVE-2026-53657
  - https://github.com/advisories/GHSA-2j9v-p4xj-cjw2
  - https://www.openssh.org/releasenotes.html
  - https://news.ycombinator.com/item?id=27941108
  - https://news.ycombinator.com/item?id=24964966
  - https://simonwillison.net/2021/Jul/13/unix-domain-sockets/
  - https://copyconstruct.medium.com/file-descriptor-transfer-over-unix-domain-sockets-dcbbf5b3b6ec
  - https://github.com/Dev916/a2a
  - https://github.com/Dev916/understudy
  - https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/ucred.h
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/socket.h
---

# Unix-Socket IPC and Singleton Local Daemons

Purpose: the operating-system facts and the resulting discipline for a local daemon that a CLI (or an editor, an agent harness, a console) talks to over a Unix domain socket, on macOS and Linux. The reference stack is Rust with tokio, but every load-bearing fact is a kernel or libc fact and the Python version is shown too. This complements `rust-async-cancellation-graceful-shutdown.md` (what happens inside the daemon when it stops) and the a2a repo profile (a production instance of everything here). Child-process supervision (process groups, PID-reuse witnesses, reaping) is a separate backlog topic and is only touched where the daemon's own lifecycle needs it.

Citations use `[n]` keyed to the `sources` list in the frontmatter. Quotes are verbatim from the cited page. Claims marked **(verified in this run)** come from the experiments in Appendix A, run on 2026-11-02 on macOS (Darwin 25.6.0, Python 3.9.6) and Linux (kernel 7.2.12, `python:3.13-slim` container, Python 3.13.18); the Rust example compiled against rustc 1.99.0, tokio 1.54.1 and rustix 1.1.6. The `x` provider returned HTTP 400 on every query and was skipped.

## 1. The invariant: one lifetime lock; the socket is derived state

A local daemon has two things that look like "I am the instance": a lock and a listening socket. Only the lock can decide ownership, because only the lock has the right failure semantics.

**flock locks die with the process, pidfiles do not.** Linux `flock(2)`: "Locks created by flock() are associated with an open file description (see open(2)). This means that duplicate file descriptors (created by, for example, fork(2) or dup(2)) refer to the same lock", and "the lock is released either by an explicit LOCK_UN operation on any of these duplicate file descriptors, or when all such file descriptors have been closed" [2]. Process death, including `SIGKILL`, closes every descriptor, so there is no stale-lock analogue of the stale pidfile **(verified in this run: a child holding `LOCK_EX` was `SIGKILL`ed and the parent's non-blocking lock then succeeded on both OSes)**. macOS documents the same model: "Locks are on files, not file descriptors. That is, file descriptors duplicated through dup(2) or fork(2) do not result in multiple instances of a lock, but rather multiple references to a single lock. If a process holding a lock on a file forks and the child explicitly unlocks the file, the parent will lose its lock" [7]. systemd's own SysV recipe concedes that a pidfile, if you must have one, "must be implemented in race-free fashion" [14]; a lock held for the process lifetime simply removes the requirement.

**Why flock (or an OFD lock) and not a POSIX record lock.** `fcntl(2)` record locks have the trap that "If a process closes any file descriptor referring to a file, then all of the process's locks on that file are released, regardless of the file descriptor(s) on which the locks were obtained" [3], and "Record locks are not inherited by a child created via fork(2), but are preserved across an execve(2)" [3], so a daemonizing fork drops them. Linux's open-file-description locks (`F_OFD_SETLK`, "Linux-specific, and available since Linux 3.15") "are inherited across fork(2) ... and are only automatically released on the last close of the open file description, instead of being released on any close of the file" [3], which is the flock model with record-lock ranges. On Linux the two families do not interact: "there is no interaction between the types of lock placed by flock() and fcntl(2)" [2]. Two more flock facts matter for daemons: "A process may hold only one type of lock (shared or exclusive) on a file", and conversion "is not guaranteed to be atomic: the existing lock is first removed, and then a new lock is established" [2]. Over NFS, flock is emulated "as fcntl(2) byte-range locks on the entire file" since Linux 2.6.12 [2]; keep the lock file on local storage.

**Advisory means cooperative.** "Advisory locks allow cooperating processes to perform consistent operations on files, but do not guarantee consistency (i.e., processes may still access files without using advisory locks possibly resulting in inconsistencies)" [7]. Every instance of your daemon must take the same lock on the same path; the lock protects against your own second copy, not against a stranger.

**The lock before everything, including the fork.** a2a's broker pins the order and explains why in its module doc: "The lock is acquired before the first `fork` and the descriptor is carried through both of them, so there is no instant at which the home is unowned: `flock` locks the open file *description*, which `fork` shares rather than copies, and the intermediate parents leave through `_exit` so no `Drop` can unlock it on the way out. A socket-existence probe cannot do this job — two daemons cold-starting in the same millisecond both see no socket" (`src/commands/broker_cmd.rs`) [75]. The lock is a sibling file (`broker.lock`), not the socket itself, because the socket path is created by `bind()` and removed by `unlink()` during the very window the lock has to cover.

Rust helpers: `fd-lock` wraps the same primitive as RAII guards and repeats the advisory warning [27]; the compiled example in section 10 uses `rustix::fs::flock` directly, and a2a uses `nix::fcntl::Flock` with `LockExclusiveNonblock`, mapping `EWOULDBLOCK` to "someone else owns it, exit quietly" [75].

## 2. Startup sequence and stale sockets

**The kernel never removes a pathname socket, and `bind()` fails on any existing path.** unix(7): "Binding to a socket with a filename creates a socket in the filesystem that must be deleted by the caller when it is no longer needed (using unlink(2))" [1]; BSD/macOS: "This file is not removed when the socket is closed -- unlink(2) must be used to remove the file" [8]. `EADDRINUSE` is documented as "The specified local address is already in use or the filesystem socket object already exists" [1], so after any crash the next `bind()` fails even though nobody listens **(verified in this run: on both OSes `bind()` after a listener closed without unlinking returned `EADDRINUSE`, and `connect()` to that path returned `ECONNREFUSED`)**. connect(2) defines the probe signal: "ECONNREFUSED: A connect() on a stream socket found no one listening on the remote address" [4].

**Unlinking without the lock is a race, and the mainstream libraries admit it.** Go's `UnixListener.close()` carries the comment "There's a race here--we can't know for sure whether someone else has come along and replaced our socket name already" [32]. Caddy unlinks the path unconditionally before binding and notes "there is a race, as the comment in net.UnixListener.close() explains" [34]. Python 3.13's `asyncio.start_unix_server(..., cleanup_socket=True)` removes the file "when the server is closed, unless the socket has been replaced after the server has been created" [35], which is the careful version of the same cleanup, but it still does nothing on a crash. Node states the general rule: "a Unix domain socket will be visible in the file system and will persist until unlinked" [37]. Neither `std::os::unix::net::UnixListener` nor `tokio::net::UnixListener` unlinks on drop **(verified in this run: the path still existed after each was dropped)**; downstream wrappers exist for exactly this reason (moq-native: "A UnixListener does not remove its filesystem node when closed" [28]; crosvm ships an `UnlinkUnixListener` [29]).

**A refused connect does not always mean stale.** On macOS, a live listener whose backlog is full refuses new connections **(verified in this run: `listen(1)`, then two `connect()`s with nothing accepted: the second returned `ECONNREFUSED` on macOS, while Linux accepted two and returned `EAGAIN` on the third)**. A daemon that probes the socket and unlinks on `ECONNREFUSED` without holding the lock will, under a connection burst on macOS, delete a live daemon's socket out from under it. Holding the lock first makes the probe safe: if you have the lock, no live instance of your daemon exists, so whatever is at the path is either stale or not yours.

**The sequence**, which the examples in section 10 implement:

1. Create the home directory `0700` (section 4) and open the lock file `0600`.
2. `flock(LOCK_EX | LOCK_NB)`. `EWOULDBLOCK` means another instance owns the home: become a client, or exit 0 ("twenty concurrent cold starts must converge on one daemon and nineteen quiet exits" [75]).
3. Holding the lock, `connect()` to the socket path. `ENOENT`: nothing there. `ECONNREFUSED`: stale, `unlink()` it. Success: a listener is live without the lock, which is a foreign process or a bug; refuse to steal the path.
4. `bind()`, fix the mode (section 4), `listen()`.
5. On shutdown, reverse: stop accepting, drain, `unlink()` the socket, remove the pidfile if any, and let the lock drop last, because "a lock released early lets a successor start into a home this daemon has not finished leaving" [75]. Cleanup is best effort; step 3 is what makes a missed cleanup harmless.

The client side mirrors it: treat `NotFound` and `ConnectionRefused` as "absent" (a2a's `is_absent` [75]), autostart the daemon at most once per code path, and then wait for the lock to be free before a respawn, "because the socket disappearing is not the same event as the daemon exiting" [75]. understudy's console bus does the lighter version for a single-user session: `socket_accepts()` probes with a timeout, a refused connect reads as litter, and a successful one raises `SessionAlreadyRunning` because "Unlinking it out from under a running console is silent theft, so the second session loses instead" (`understudy/bus.py`) [76]. That design has no lock, so it inherits the macOS full-backlog caveat above; it is acceptable there because one human starts one session.

## 3. Where the socket lives

**Path length.** "On Linux, sun_path is 108 bytes in size" and portable code should remember that "some implementations have sun_path as short as 92 bytes" [1]; BSD and macOS allow "at most 104 characters" [8] (`char sun_path[104]` in Apple's `sys/un.h` [9]). Node's docs give the usable string lengths: "Typical values are 107 bytes on Linux and 103 bytes on macOS" [37] **(verified in this run: a 107-byte path bound on Linux, 108 bytes failed with "AF_UNIX path too long")**. understudy refuses paths over 103 bytes at startup [76]. The limit applies to the string handed to the kernel, not to the resolved path, so a long home directory can be escaped by `chdir()` and binding a relative name, which brokk-mj-core does [30] and which QEMU adopted after hitting the macOS limit [31]. Deep per-user caches (`~/Library/Application Support/<Vendor>/<App>/...`, Nix store paths, CI workspaces) hit this limit in practice.

**Directory.** On Linux the specified home is `$XDG_RUNTIME_DIR`, which "defines the base directory relative to which user-specific non-essential runtime files and other file objects (such as sockets, named pipes, ...) should be stored"; its "Unix access mode MUST be 0700", its lifetime "MUST be bound to the user being logged in", and the spec requires AF_UNIX sockets and file locking to work there [53]. Files in it "MAY be subjected to periodic clean-up" unless their access time is touched every six hours or the sticky bit is set [53]; a daemon that idles for days should touch its lock file or use a subdirectory it re-creates. macOS has no `$XDG_RUNTIME_DIR`; the equivalents are a `0700` directory under `$TMPDIR` (per-user on macOS) or under the application's own home. a2a uses a `0700` `~/.a2a` home for all of lock, socket, pidfile and logs [75].

**Not a predictable name in `/tmp`.** The sticky bit stops other users deleting or renaming your entry; it does not stop them creating the name first. An attacker who pre-creates `/tmp/<app>.sock` as a file, symlink or directory denies service or redirects the daemon, and "an attacker can create a directory with no delete permissions to make it impossible to create a socket" [58][59]. The tools that live in `/tmp` do so inside a per-user directory they own: tmux's "tmux-UID" directory "is created by tmux and must not be world readable, writable or executable" [52], and ssh-agent until recently used a randomised `$TMPDIR/ssh-XXXXXXXXXX/` directory. OpenSSH 10.1 (2025-10-06) moved further: "move agent listener sockets from /tmp to under ~/.ssh/agent for both ssh-agent(1) and forwarded sockets in sshd(8)", because "This ensures processes that have restricted filesystem access that includes /tmp do not ambiently have the ability to use keys in an agent"; `-T` "makes ssh-agent put the socket back in /tmp" [70]. The current man page documents the default as "a random path matching $HOME/.ssh/agent/s.*" [51] **(confirmed on the macOS OpenSSH 10.3p1 man page)**.

**Linux abstract namespace.** "An abstract socket address is distinguished (from a pathname socket) by the fact that sun_path[0] is a null byte"; "Socket permissions have no meaning for abstract sockets"; "Abstract sockets automatically disappear when all open references to the socket are closed. The abstract socket namespace is a nonportable Linux extension" [1] **(verified in this run: bind, close, rebind with no unlink)**. systemd spells it with a leading `@` ("The "@" is replaced with a NUL character before binding") [11] and Node with a leading `\0` [37]; Rust exposes it as `std::os::linux::net::SocketAddrExt::from_abstract_name`, stable since 1.70.0 [19]. Abstract sockets trade the stale-file problem for the loss of filesystem access control: any process in the same network namespace can connect, so peer-credential checks (section 5) become mandatory rather than defence in depth. They are not available on macOS, so a cross-platform daemon still needs the pathname discipline.

## 4. Permissions: the directory is the portable gate

**What the socket mode does.** unix(7): "On Linux, connecting to a stream socket object requires write permission on that socket; sending a datagram to a datagram socket likewise requires write permission", and creating one needs "write and search (execute) permission on the directory" [1]; connect(2) lists `EACCES` as "Write permission is denied on the socket file, or search permission is denied for one of the directories in the path prefix" [4]. Only the write bit matters: PostgreSQL's documentation says "for a Unix-domain socket, only write permission matters" [50], and FreeBSD unix(4) that "the destination of a connect(2) or sendto(2) must be writable" [8].

Measured **(verified in this run)**:

| Socket mode | Directory mode | Connecting as another uid, Linux | Connecting as owner, macOS |
|---|---|---|---|
| 0600 | 0711 | `EACCES` | ok |
| 0644 | 0711 | `EACCES` | n/a |
| 0622 | 0711 | connects | n/a |
| 0666 | 0711 | connects | n/a |
| 0666 | 0700 | `EACCES` | n/a |
| 0000 or 0444 | 0700 | n/a | `EACCES` |
| 0200 | 0700 | n/a | ok |
| 0666 | 0600 (no search bit) | n/a | `EACCES` |

So current macOS enforces socket modes too, and both kernels enforce directory search permission.

**But POSIX does not promise any of it.** "POSIX does not make any statement about the effect of the permissions on a socket file, and on some systems (e.g., older BSDs), the socket permissions are ignored. Portable programs should not rely on this feature for security" [1]. PostgreSQL names Solaris 10 and later as a system where the mode is ignored and recommends restricting search permission on the socket directory instead [50]. The portable control is therefore the `0700` directory; the socket mode is defence in depth; and the peer check in section 5 is the inner gate that holds even when the first two are misconfigured (a2a: "the 0700 home is the outer gate, and the peer-uid check is the inner one" [75]).

**bind() honours the umask.** With the default `umask 022` the socket file is created `0755` **(verified in this run on both OSes)**, which on Linux is not connectable by others only because the write bit is clear; a `umask 000` would leave it `0777`. FreeBSD documents `fchmod()` before `bind()` as the race-free fix ("This avoids the race that would otherwise occur between creation of the file and a subsequent call to chmod(2)") [8], but tinc found that "fchmod(fd, 0777) has no effect on Unix domain sockets on some systems" and switched to setting the umask around `bind()` [60], and Tor filed the same problem [61]. The two portable options are `umask` around `bind()` (the Python example) or `bind()` then `chmod()` inside a `0700` directory, where the window is closed by the directory (the Rust example and a2a [75]).

**Group-based access is a deliberate trade.** Docker: "By default, a unix domain socket (or IPC socket) is created at /var/run/docker.sock, requiring either root permission, or docker group membership" [42]; "The docker group grants root-level privileges to the user" [40], because a member "can start a container where the /host directory is the / directory on your host; and the container can alter your host filesystem without any restriction" [41]. Mounting `docker.sock` into a container is a standard escape path [43]. PostgreSQL's `unix_socket_permissions` default is `0777`, "meaning anyone can connect", with `0770` and `0700` as the documented alternatives [50]. A group socket is a documented statement that group membership equals the daemon's privilege; if your daemon runs as the user who talks to it, there is no reason to widen beyond `0600`.

**Two incidents from the same mistake.** CVE-2015-8222: "The lxd-unix.socket systemd unit file in the Ubuntu lxd package before 0.20-0ubuntu4.1 uses world-readable permissions for /var/lib/lxd/unix.socket, which allows local users to gain privileges via unspecified vectors" [56], a `SocketMode=` left at its `0666` default (section 8). CVE-2026-53657 (Lima, published 2026-07-10, CVSS 3.1 8.2): "an arbitrary user in the VM could access /run/lima-guestagent.sock when the guest agent is enabled, which could result in running arbitrary commands with root privileges in the VM because the guest agent socket provides tunneling for arbitrary addresses, including Unix socket addresses for privileged daemons like D-Bus. This issue is fixed in version 2.1.3" [68][69]; the fix was an `os.Chown` plus `os.Chmod(socket, 0o600)` [57].

## 5. Peer identity: trust the uid, not the pid

Every Unix kernel will tell a server who connected, and the answer is unforgeable in a way that nothing inside the byte stream is.

| Mechanism | Platform | Returns | Snapshot taken | Source |
|---|---|---|---|---|
| `getsockopt(SOL_SOCKET, SO_PEERCRED)` | Linux | `struct ucred { pid, uid, gid }` | "at the time of the call to connect(2), listen(2), or socketpair(2)" | [1] |
| `getpeereid(3)` | macOS, FreeBSD, OpenBSD, NetBSD | effective uid and gid | peer's `connect(2)` (server side) or `listen(2)` (client side) | [6] |
| `getsockopt(SOL_LOCAL, LOCAL_PEERCRED)` | macOS, FreeBSD | `struct xucred { cr_version, cr_uid, cr_ngroups, cr_groups[16] }`; FreeBSD's also carries `cr_pid`, macOS's does not | peer's `connect(2)` | [8][9][78] |
| `LOCAL_PEERPID`, `LOCAL_PEEREPID` | macOS | pid, effective pid | connect time; subject to PID reuse | [9] |
| `LOCAL_PEERTOKEN` | macOS 13+ | audit token, "securely bound to the original connecting process, even if that process exits and its PID is reused" | connect time | [10] |
| `SO_PEERPIDFD` | Linux 6.5+ | a pidfd for the peer, "which allows programmers not to care about PID reuse problem" | connect time | [67][48] |
| `SCM_CREDENTIALS` ancillary message | Linux | per-message `ucred`; "the SO_PASSCRED option must be enabled on the socket" | per message | [1] |

getpeereid's man page states the guarantee precisely: "This mechanism is reliable; there is no way for either side to influence the credentials returned to its peer except by calling the appropriate system call (i.e., either connect(2) or listen(2)) under different effective credentials" [6]. **(Verified in this run: `LOCAL_PEERCRED` on macOS returned `cr_version 0`, the connecting uid and 16 groups; `LOCAL_PEERPID` returned the connecting pid; `SO_PEERCRED` on Linux returned pid, uid and gid.)**

**The pid is not an identity.** It is a connect-time snapshot, and the process may have exited and its pid been reused by the time you act on it. The kernels say so by what they shipped: Linux 6.5 added `SCM_PIDFD` and `SO_PEERPIDFD` ("SO_PEERPIDFD allows to get pidfd of peer socket holder pidfd, analogous of SO_PEERCRED which allows to get plain PID") [67]; dbus 1.15.8 exposes it as `ProcessFD` in `GetConnectionCredentials()` [46][47]; systemd moved its internal process tracking to pidfds in v256 and Polkit v124 uses them [48]; and Apple's DTS explains that "Protecting against PID reuse is one of the primary reasons that audit tokens exist" and that `LOCAL_PEERTOKEN` is the supported answer, with the caveat that an inherited or passed descriptor carries the original connector's token [10]. Even a pidfd can refer to a process that is already gone: "A pidfd can outlive the task it refers to, and thus user-space must already be prepared that the task underlying a pidfd is gone at the time they get their hands on the pidfd" [49]. Authorize on uid (and gid), log the pid for diagnostics, and if you must act on a process use a pidfd or audit token.

**Language bindings.** Rust std's `UCred` maps platforms to calls: Linux, Android and Cygwin use `SO_PEERCRED`; the BSDs use `getpeereid`; Apple uses `getpeereid` plus `getsockopt(SOL_LOCAL, LOCAL_PEERPID)` for the pid; the API is still unstable (`peer_credentials_unix_socket`) and documents uid/gid as "the effective UID/GID of the process at the domain socket's endpoint" with `pid: Option<pid_t>` because "the PID part of the peer credentials is not supported on every platform" [18]. tokio 1.54.1 stabilizes the same shape: `UnixStream::peer_cred()` "Returns effective credentials of the process which called connect or pair" [24], and `UCred::pid()` is `Some` on "Linux, Android, OpenBSD, FreeBSD (since FreeBSD 13), NetBSD, NTO, iOS, macOS, tvOS, watchOS, visionOS, Solaris, Illumos, Cygwin, Haiku, and Redox" [25]. a2a does it by hand with `nix`: `sockopt::PeerCredentials` on Linux and `sockopt::LocalPeerCred` elsewhere, then "Peer uid == our uid" with the justification that "File modes are advisory on some platforms and nonexistent on others; `LOCAL_PEERCRED`/`SO_PEERCRED` asks the kernel who" (`src/broker/sockserv.rs`) [75].

**D-Bus as the reference protocol.** D-Bus's EXTERNAL mechanism "is the recommended authentication mechanism on platforms where credentials can be transferred out-of-band, in particular Unix platforms that can perform credentials-passing over the unix: transport"; the client sends "the ASCII decimal string form of the integer Unix user ID as the authorization identity, for example 1000" and the server checks it against the kernel's answer [44]. libdbus deliberately skips `SCM_CREDENTIALS` on Linux because "the SO_PEERCRED socket option is equally informative" [45]. Practitioners use the same property for local role-based access without passwords or certificates "because if the user is logged in on the system, they are already authorized" (HN comment, [71]).

## 6. Framing a protocol over the stream

**JSON Lines.** The format rules: UTF-8 and "a byte order mark (U+FEFF) must NOT be included"; each line is one JSON value and "null is a valid value but a blank line is not"; the "Line Terminator is '\n'" with `\r\n` tolerated; a trailing newline is "strongly recommended but not required" [39]. a2a's wire format, understudy's bus and every agent harness in this corpus use it because a human can `nc -U` the socket and read the traffic.

**Bound the line, or a client can exhaust memory.** tokio-util's `LinesCodec::new()` "will not have an upper bound on the length of a buffered line"; `new_with_max_length` is the one to use on a socket other processes can reach, and on overflow it returns `MaxLineLengthExceeded` and then discards until the next newline [26]. Python's `asyncio` streams take a `limit` (the example in section 10 sets it to the same `MAX_LINE`), and the hand-rolled Rust reader uses `take(MAX_LINE).read_until(b'\n')` and closes the connection when the buffer fills without a newline. Ancillary data has its own kernel bound: "The maximum ancillary buffer size allowed per socket can be set using /proc/sys/net/core/optmem_max" [5].

**Correlation and versioning.** Put a request `id` in every line so replies can be matched when a client pipelines, and start every connection with a hello that carries a protocol version. a2a refuses a mismatched `protocol_version` with a remedy string ("`a2a broker restart`") rather than guessing [75]; the realistic failure is a CLI upgraded under a daemon that is still running the old binary.

**SIGPIPE.** On Linux, writing to a socket whose peer has gone raises `EPIPE` and "If enabled, a SIGPIPE is sent as well. This can be avoided by passing the MSG_NOSIGNAL flag to send(2) or sendmsg(2)" [1]. macOS has no `MSG_NOSIGNAL`; the per-socket option is `SO_NOSIGPIPE`, which "prevents SIGPIPE from being raised when a write fails on a socket to which there is no reader; instead, the write to the socket returns with the error EPIPE" [77][79] (the same conclusion reached independently by php-src, questdb-rs and corosio [62][63][64]). Rust programs are covered by default: the runtime "set[s] SIGPIPE to ignore when the program starts up in order to prevent this problem" [20], and the behaviour is opt-out via `-Zon-broken-pipe` or the unstable `#[unix_sigpipe]` attribute [21]; note that Rust resets `SIGPIPE` to default in children it spawns [22], so a helper process that writes to a closed pipe still dies the Unix way. Python and Node install their own handling; a C daemon must call `signal(SIGPIPE, SIG_IGN)` or set the socket option itself.

## 7. Passing file descriptors (SCM_RIGHTS)

A daemon that owns a pty, a log file or a listening socket can hand the descriptor to a client; the kernel duplicates it into the receiver's table. The rules from unix(7) and cmsg(3): "At least one byte of real data should be sent when sending ancillary data. On Linux, this is required to successfully send ancillary data over a UNIX domain stream socket" [1]; "SCM_MAX_FD has the value 253 (or 255 before Linux 2.6.38)" [1]; "For portability, ancillary data should be accessed using only the macros described here", with the buffer "wrapped in a union in order to ensure it is suitably aligned" [5]. D-Bus turned the one-byte rule into protocol: "Immediately after connecting to the server, the client must send a single nul byte. This byte may be accompanied by credentials information on some operating systems that use sendmsg() with SCM_CREDS or SCM_CREDENTIALS" [44].

The cmsg API is sharp. One Android engineer's list of the ways it goes wrong (HN comment, [72]): not aligning the buffer, leaking descriptors when more arrive than expected, dereferencing `CMSG_DATA` without checking the header, `CMSG_SPACE(fd_count)` instead of `CMSG_SPACE(fd_count * sizeof(int))`, mixing up `CMSG_SPACE` and `CMSG_LEN` for `cmsg_len` versus `msg_controllen`, and lengths in descriptors instead of bytes. Use a library (`nix::sys::socket::sendmsg` with `ControlMessage::ScmRights`, Python's `socket.send_fds` since 3.9, Go's `syscall.UnixRights`) rather than the raw macros, and treat a received descriptor count that differs from the expected one as a protocol error. The classic use cases are zero-downtime listener handover and letting a privileged helper open a file the client cannot [74][72].

## 8. Socket activation: let the init own the path

Socket activation removes the stale-socket and race problems by construction: the init system creates the socket before the daemon exists, owns the path for the daemon's whole life, and passes a ready listener in. daemon(7) adds that with it "all providers and all consumers of the sockets" can start concurrently, "the socket stays bound and accessible during the restart", and "For developing a new-style daemon, none of the initialization steps recommended for SysV daemons need to be implemented" [14]. Docker supports it with `dockerd -H fd://` [42]; rootless Podman's endpoint is `unix://$XDG_RUNTIME_DIR/podman/podman.sock`, delivered by a `systemd --user` socket unit [54][55].

### systemd (user unit)

```ini
# ~/.config/systemd/user/myd.socket
[Socket]
ListenStream=%t/myd/myd.sock
SocketMode=0600
DirectoryMode=0700
Accept=no
RemoveOnStop=yes

[Install]
WantedBy=sockets.target
```

```ini
# ~/.config/systemd/user/myd.service
[Service]
ExecStart=%h/.local/bin/myd --activated
# no Type=forking: an activated daemon must not daemonize
```

The facts behind each line: an address that "starts with a slash ("/")" is "a file system socket in the AF_UNIX socket family" [11]; `%t` "is either /run/ (for the system manager) or the path $XDG_RUNTIME_DIR resolves to (for user managers)" [13], and user units load from "$XDG_CONFIG_HOME/systemd/user or $HOME/.config/systemd/user" [13]; `SocketMode=` "Defaults to 0666" [11], which is how CVE-2015-8222 happened, so set it; `DirectoryMode=` "Defaults to 0755" [11]; `Accept=no` (the default) means "all listening sockets themselves are passed to the started service unit" rather than one service per connection [11]; `RemoveOnStop=yes` means "any file nodes created by this socket unit are removed when it is stopped" [11]; `SocketUser=`/`SocketGroup=` default to the invoking user in user context [11]. In the daemon, `sd_listen_fds(3)` returns the count of passed descriptors starting at `SD_LISTEN_FDS_START` (3), returns 0 "If $LISTEN_FDS/$LISTEN_PID was not set or was not correctly set for this daemon", checks that `$LISTEN_PID` matches `getpid()`, sets `FD_CLOEXEC` on every passed descriptor, and with `unset_environment` clears "$LISTEN_FDS, $LISTEN_PID, $LISTEN_PIDFDID, and $LISTEN_FDNAMES" [12]. In Rust, `tokio::net::UnixListener::from_std(std::os::unix::net::UnixListener::from_raw_fd(3))` after the `LISTEN_PID` check is the whole integration; the `listenfd` and `sd-notify` crates wrap it.

### launchd (user agent)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.example.myd</string>
  <key>ProgramArguments</key>
  <array><string>/usr/local/bin/myd</string><string>--activated</string></array>
  <key>Sockets</key>
  <dict>
    <key>Listener</key>
    <dict>
      <key>SockPathName</key><string>/Users/me/Library/Application Support/myd/myd.sock</string>
      <key>SockPathMode</key><integer>384</integer>
    </dict>
  </dict>
</dict>
</plist>
```

`Sockets` "is used to specify launch on demand sockets that can be used to let launchd know when to run the job. The job must check-in to get a copy of the file descriptors using the launch_activate_socket(3) API" [15]. `SockPathName` "implies SockFamily is set to "Unix". It specifies the path to connect(2) or bind(2) to"; `SockPathMode` "specifies the mode of the socket. Known bug: Property lists don't support octal, so please convert the value to decimal" [15], hence `384` for `0600`; `SockType` defaults to stream and `SockPassive` to true (listen) [15]. `SecureSocketWithKey` is the variant where "a securely generated socket is created and the path is assigned to the environment variable that is inherited by all jobs spawned in the job's context" [15], which is how `SSH_AUTH_SOCK` is provided on macOS. `launch_activate_socket(const char *name, int **fds, size_t *cnt)` fills an array of all descriptors launchd created for that `Sockets` key; the man page describes it as the way "a launchd(8) job [can] retrieve a set of file descriptors corresponding to a socket service that launchd(8) has created and advertised on behalf of the job" [16][17], and it is the only correct way, since launchd does not promise descriptor numbers [17]. A launchd job must not "Call daemon(3)" nor "Do the moral equivalent of daemon(3) by calling fork(2) and have the parent process exit(3)" [15]. The plist above passes `plutil -lint` **(verified in this run)**.

**When to choose which.** Activation is right when the daemon should outlive any single CLI invocation, start on first connect, and be restarted by the init on crash. The self-claiming daemon of section 2 is right when the tool must work in a plain shell, a CI runner or a container with no init, and when the user never installs anything. a2a ships the second and documents that the lock, not launchd, is its singleton [75]; a tool that wants both can accept an `--activated` flag and skip steps 1 to 4 when `LISTEN_FDS` or a launchd check-in succeeds.

## 9. Lifecycle of a lazily started daemon-lite

When the CLI starts the daemon itself, the classic steps are daemon(7) 5 to 11: fork; "In the child, call setsid() to detach from any terminal and create an independent session"; "In the child, call fork() again, to ensure that the daemon can never re-acquire a terminal again"; exit the first child; "connect /dev/null to standard input, output, and error"; "reset the umask to 0"; `chdir("/")` [14]. a2a's version keeps stdout and stderr on a log file instead of `/dev/null`, writes its own pid to a `0600` pidfile after daemonizing ("Ops needs a pid it can signal, and the crash tests need one they can `SIGKILL`"), and ignores `SIGHUP` between the two forks [75]. Do not reset the umask to 0 if you then `bind()` a socket (section 4).

glibc 2.26 added `POSIX_SPAWN_SETSID` so a spawned child can start in its own session without a fork in the parent [65][66]; Apple's `spawn.h` does not define it **(verified in this run: no such symbol in the macOS SDK header)**, so a cross-platform launcher still double-forks or runs `setsid` in the child before `exec`. Idle exit (a2a's `IdleWatch`) is what keeps a lazily started daemon from accumulating: when no client has connected for the idle window, run the teardown in section 2 step 5 and exit, and let the next CLI invocation re-claim. Everything about stopping cleanly inside the process, bounded drain and cancellation safety, is in `rust-async-cancellation-graceful-shutdown.md`.

## 10. Reference implementation

Both programs implement sections 1 to 6: `0700` home, lifetime `flock`, lock-first stale probe, `0600` socket, peer-uid check, bounded JSONL. The Rust one compiled with rustc 1.99.0 against tokio 1.54.1, rustix 1.1.6 and serde_json 1.0.154 **(verified in this run)**; the Python one ran its self-test on Python 3.13.18 (second `claim()` returned `None` while the daemon lived; a request got `{"ok":true}`; after `SIGKILL` the socket file remained and the next `claim()` took the lock and unlinked it) and `claim()` alone on macOS Python 3.9.6 **(verified in this run)**.

```rust
//! Singleton local daemon over a Unix socket: lock, then stale-probe, then bind.
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use rustix::fs::{flock, FlockOperation};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// Longest request line we will buffer before closing the connection.
const MAX_LINE: u64 = 64 * 1024;

/// Field order is drop order: the listener goes first, the lock last, and
/// `Drop` unlinks the socket path in between.
pub struct Singleton {
    listener: UnixListener,
    sock: PathBuf,
    _lock: File,
}

impl Drop for Singleton {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.sock);
    }
}

/// `Ok(None)` means another instance holds the lock: exit quietly.
pub fn claim(home: &Path) -> io::Result<Option<Singleton>> {
    // 1. The home directory is the outer gate: 0700, so no other uid can even
    //    reach the socket path. (Harmless if it already exists with that mode.)
    fs::DirBuilder::new().recursive(true).mode(0o700).create(home)?;

    // 2. The lock decides ownership. It lives for the whole process and the
    //    kernel releases it on any exit, including SIGKILL.
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .mode(0o600)
        .open(home.join("daemon.lock"))?;
    match flock(&lock, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => {}
        Err(rustix::io::Errno::WOULDBLOCK) => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    // 3. Holding the lock, a leftover socket file can only be stale, so a
    //    refused connect is the signal to unlink it. A connect that succeeds
    //    means someone is listening without the lock: refuse to steal it.
    let sock = home.join("daemon.sock");
    match std::os::unix::net::UnixStream::connect(&sock) {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "a listener is live on the socket but does not hold the lock",
            ))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => fs::remove_file(&sock)?,
        Err(e) => return Err(e),
    }

    // 4. bind() honours the umask, so tighten the mode right after. The
    //    window is covered by the 0700 directory and the peer check below.
    let listener = UnixListener::bind(&sock)?;
    fs::set_permissions(&sock, fs::Permissions::from_mode(0o600))?;

    Ok(Some(Singleton { listener, sock, _lock: lock }))
}

pub async fn serve(daemon: Singleton) -> io::Result<()> {
    let me = rustix::process::getuid().as_raw();
    loop {
        let (stream, _) = daemon.listener.accept().await?;
        // Ask the kernel who connected; never trust a uid the client sends.
        // (SO_PEERCRED on Linux, LOCAL_PEERCRED on macOS, behind one tokio API.)
        match stream.peer_cred() {
            Ok(cred) if cred.uid() == me => {
                tokio::spawn(handle(stream));
            }
            Ok(_) | Err(_) => drop(stream),
        }
    }
}

/// One request per line, JSON, bounded; one JSON line back per request.
async fn handle(stream: UnixStream) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = Vec::with_capacity(1024);
    loop {
        line.clear();
        let n = (&mut reader).take(MAX_LINE).read_until(b'\n', &mut line).await?;
        if n == 0 {
            return Ok(()); // peer closed
        }
        if line.last() != Some(&b'\n') {
            // Hit MAX_LINE without a newline: the peer is misbehaving.
            return Err(io::Error::new(io::ErrorKind::InvalidData, "request line too long"));
        }
        let reply = match serde_json::from_slice::<serde_json::Value>(&line) {
            Ok(req) => serde_json::json!({ "id": req.get("id"), "ok": true }),
            Err(e) => serde_json::json!({ "id": null, "ok": false, "error": e.to_string() }),
        };
        let mut out = serde_json::to_vec(&reply)?;
        out.push(b'\n');
        writer.write_all(&out).await?; // Rust ignores SIGPIPE at startup, so a
                                       // dead peer surfaces here as EPIPE.
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let home = std::env::var_os("DAEMON_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("uds-scratch"));
    let Some(daemon) = claim(&home)? else {
        eprintln!("already running");
        return Ok(());
    };
    tokio::select! {
        r = serve(daemon) => r,
        _ = tokio::signal::ctrl_c() => Ok(()),
    }
}
```

`Cargo.toml` features used: `tokio = { features = ["rt-multi-thread", "macros", "net", "io-util", "signal", "time", "sync"] }`, `rustix = { features = ["fs", "net", "process"] }`. Not shown: the hello/version line (section 6), the idle timer (section 9), and `--activated` (section 8).

```python
"""Singleton local daemon over a Unix socket: lock, then stale-probe, then bind."""
import asyncio, fcntl, os, socket, struct, sys
from pathlib import Path

MAX_LINE = 64 * 1024


def claim(home: Path):
    """Return (lock_file, socket_path) if we own the daemon role, else None."""
    home.mkdir(mode=0o700, parents=True, exist_ok=True)   # outer gate
    lock = open(home / "daemon.lock", "a")
    os.chmod(lock.fileno(), 0o600)
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)  # the singleton
    except BlockingIOError:
        lock.close()
        return None
    sock = home / "daemon.sock"
    probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        probe.connect(str(sock))
        raise RuntimeError("a listener is live on the socket but does not hold the lock")
    except FileNotFoundError:
        pass
    except ConnectionRefusedError:
        sock.unlink()                                      # stale, and we hold the lock
    finally:
        probe.close()
    return lock, sock


def peer_uid(sock: socket.socket) -> int:
    """Effective uid of the peer, from the kernel."""
    if sys.platform == "linux":
        _pid, uid, _gid = struct.unpack("3i", sock.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12))
        return uid
    # macOS / BSD: struct xucred { u_int cr_version; uid_t cr_uid; short cr_ngroups; gid_t cr_groups[16]; }
    return struct.unpack_from("Ii", sock.getsockopt(0, socket.LOCAL_PEERCRED, 76))[1]


async def handle(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    if peer_uid(writer.get_extra_info("socket")) != os.getuid():
        writer.close()
        return
    try:
        while line := await reader.readline():          # bounded by the server's `limit`
            if not line.endswith(b"\n"):
                break
            writer.write(b'{"ok":true}\n')
            await writer.drain()
    except (asyncio.LimitOverrunError, ValueError):
        pass
    finally:
        writer.close()


async def serve(home: Path):
    claimed = claim(home)
    if claimed is None:
        print("already running", flush=True)
        return
    lock, sock = claimed
    old = os.umask(0o077)                                  # bind() honours the umask
    try:
        server = await asyncio.start_unix_server(handle, str(sock), limit=MAX_LINE,
                                                 cleanup_socket=True)   # 3.13+
    finally:
        os.umask(old)
    sock.chmod(0o600)                                      # umask 077 leaves 0700; only the write bit matters
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(serve(Path(sys.argv[1])))
```

On Python 3.12 and earlier drop the `cleanup_socket` argument and unlink in a `finally`; `socket.LOCAL_PEERCRED` is defined on macOS builds of CPython (its value is 1 and the level `SOL_LOCAL` is 0 [9]).

## 11. Worked examples from the field

| System | Where the socket lives | Access control | Singleton / stale handling | Source |
|---|---|---|---|---|
| Docker | `/var/run/docker.sock` | root or `docker` group ("grants root-level privileges") | systemd socket activation available (`-H fd://`) | [40][41][42] |
| Podman (rootless) | `$XDG_RUNTIME_DIR/podman/podman.sock` | user-owned runtime dir | `systemd --user` socket unit, `ListenStream=%t/podman/podman.sock` | [54][55] |
| ssh-agent (OpenSSH 10.1+) | `$HOME/.ssh/agent/s.*` (random name) | owner-only; "easily abused by root or another instance of the same user" | sockets "should get automatically removed when the agent exits"; `-U` skips stale cleanup, `-u` cleans without running | [51][70] |
| tmux | `/tmp/tmux-<uid>/<name>` (or `$TMUX_TMPDIR`) | directory "must not be world readable, writable or executable" | `SIGUSR1` recreates a removed socket | [52] |
| D-Bus | `unix:path=` or `unix:abstract=` | EXTERNAL auth on kernel peer credentials | bus owns the path | [44][45] |
| PostgreSQL | `unix_socket_directories` | `unix_socket_permissions` default `0777`; directory permissions recommended where modes are ignored | postmaster lock file | [50] |
| a2a broker | `~/.a2a/broker.sock` (`0700` home) | `0600` socket, peer uid must equal daemon uid | `flock(broker.lock)` before fork, lock-then-probe, lock released last | [75] |
| understudy console bus | per-session path under the project | single-user | connect probe; refuses to unlink a live socket; no lock | [76] |

## 12. Checklist

- [ ] One `flock` (or OFD lock) on a sibling `.lock` file, taken before any fork and held until exit; `EWOULDBLOCK` means "not the daemon".
- [ ] Probe the socket only while holding the lock; `ECONNREFUSED` or `ENOENT` then means unlink and bind; a successful connect means stop.
- [ ] Socket path under 100 bytes, inside a `0700` directory you own (`$XDG_RUNTIME_DIR/<app>/` on Linux; a `0700` dir under `$TMPDIR` or the app home on macOS); never a predictable `/tmp/<app>.sock`.
- [ ] Socket mode `0600` after `bind()` (or `umask 077` around it); treat the mode as defence in depth, not the control.
- [ ] Authorize every connection on the kernel's peer uid/gid; log the pid; never authorize on it.
- [ ] JSONL with a hard maximum line length, a request `id`, and a hello carrying a protocol version.
- [ ] `SIGPIPE` ignored or `MSG_NOSIGNAL`/`SO_NOSIGPIPE` set; `EPIPE` handled as "client went away".
- [ ] For `SCM_RIGHTS`: at least one data byte, at most 253 descriptors, a library not raw macros, received-count checked.
- [ ] Teardown in reverse: stop accepting, drain, unlink socket, remove pidfile, release lock last.
- [ ] If an init system is available, prefer socket activation with `SocketMode=0600` / `SockPathMode` 384 and no daemonizing.

## 13. Common misconceptions

- *"If the socket file exists, the daemon is running."* The kernel never removes it [1][8]; after a crash it is litter that makes `bind()` fail [1].
- *"A refused connect means the socket is stale."* Only if you hold the lock. A live macOS listener with a full backlog also refuses **(verified in this run)**.
- *"The pidfile is the singleton."* It is a note the daemon wrote; the kernel does not know about it. A held `flock` is released on any exit, a pidfile is not [2][14].
- *"chmod 600 on the socket protects it."* On Linux and current macOS, yes **(verified in this run)**; POSIX promises nothing and some systems ignore it [1][50]. The directory mode is the portable control.
- *"The peer pid identifies the client."* It is a connect-time snapshot subject to reuse; `SO_PEERPIDFD` and `LOCAL_PEERTOKEN` exist because of that [67][10].
- *"A lock on the socket file itself is enough."* The socket path is unlinked and re-created during startup; lock a sibling file.
- *"Abstract sockets avoid all the problems."* They avoid stale files and path limits, at the cost of filesystem access control and macOS support [1][19].
- *"Rust's UnixListener cleans up after itself."* Neither std's nor tokio's unlinks on drop **(verified in this run)** [28][29].

## 14. Synthesis (inferred)

These are this document's own conclusions, drawn from the cited facts rather than stated by any source.

- **Ownership, liveness and reachability are three different facts and need three different mechanisms.** The lock answers "who owns this home" (kernel-released, race-free). The connect probe answers "is anyone listening" (and only that: it cannot tell stale from saturated on macOS). The socket file answers "where to connect". Designs that use one of these to infer another (socket exists therefore running; connect refused therefore stale; pidfile present therefore owned) are where the bugs in section 2 live. Make each fact come from its own mechanism, and make the lock the one that decides.
- **The macOS full-backlog refusal is an argument for lock-first on every platform**, not a macOS quirk to special-case: a probe-and-unlink daemon is correct exactly as often as its backlog is not full, which is to say under light load only.
- **Treat the socket mode as a tripwire rather than a wall.** It costs one `chmod`, it catches the world-writable mistake behind two CVEs, and when it is bypassed (a system that ignores it, an abstract socket, a descriptor passed in) the directory and the peer check still hold. The three-layer order in a2a (0700 directory, 0600 socket, uid check) is the right default for a single-user daemon; widen one layer at a time and only with a reason you can write down, the way Docker documents the `docker` group as root-equivalent.
- **A lazily started daemon should be a socket-activated daemon with a fallback, not the other way around.** The lock-first claim in section 2 is the fallback for shells, CI and containers; where launchd or `systemd --user` exists, letting the init own the path removes the whole stale-socket class and gives crash restart for free. Ship `--activated` and have `claim()` short-circuit when the init handed over a listener.
- **Version the hello before the first feature.** The failure that actually happens is a CLI upgraded under a long-lived daemon; refusing a mismatched protocol version with a one-line remedy, as a2a does, turns a confusing hang into an actionable message.

## Appendix A: experiments run for this document (2026-11-02)

Scripts were Python 3 using the `socket`, `fcntl` and `os` modules; Linux runs used `docker run --rm python:3.13-slim` (kernel 7.2.12), macOS runs used the system Python 3.9.6 on Darwin 25.6.0. Each result is quoted in the text as **(verified in this run)**.

1. Live listener, second `bind()` to the same path: `EADDRINUSE` (both).
2. Listener closed without `unlink()`: path exists; `bind()`: `EADDRINUSE`; `connect()`: `ECONNREFUSED` (both).
3. Socket mode after `bind()` under `umask 022`: `0755` (both); `chmod 0600` afterwards succeeds (both).
4. Linux, connecting as uid 65534: `0600` → `EACCES`; `0644` → `EACCES`; `0622` → connects; `0666` → connects; `0666` in a `0700` directory → `EACCES`. macOS, connecting as the owner: `0000` and `0444` → `EACCES`; `0200` → connects; `0666` in a `0600` directory → `EACCES`.
5. `LOCAL_PEERCRED` (macOS): `cr_version 0`, connecting uid, 16 groups; `LOCAL_PEERPID`: connecting pid. `SO_PEERCRED` (Linux): pid, uid, gid of the connector.
6. `flock(LOCK_EX)` held by a child, child `SIGKILL`ed: parent's `LOCK_EX|LOCK_NB` fails with `EAGAIN` while the child lives, succeeds after (both). Child opening the file anew while the parent holds the lock: `EAGAIN` (Linux).
7. Abstract namespace (Linux): bind, close, rebind without unlink succeeds.
8. `sun_path` (Linux): 107-byte path binds; 108 and 109 bytes fail with "AF_UNIX path too long".
9. `listen(1)` with nothing accepted: macOS refuses the second `connect()` with `ECONNREFUSED`; Linux accepts two and returns `EAGAIN` on the third.
10. `std::os::unix::net::UnixListener` and `tokio::net::UnixListener` dropped: socket file still present (macOS, rustc 1.99.0, tokio 1.54.1).
11. Section 10 Rust example: `cargo build` clean. Section 10 Python example: self-test passed on Linux 3.13.18 (second claim `None`; request answered; after `SIGKILL` the stale socket was unlinked by the next claim); `claim()` on macOS 3.9.6 behaved the same.
12. Section 8 launchd plist: `plutil -lint` OK. The systemd unit was not validated (no `systemd-analyze` on this machine); its directives are quoted from systemd.socket(5) and systemd.unit(5).
13. macOS SDK headers: `sys/un.h` defines `LOCAL_PEERCRED 0x001`, `LOCAL_PEERPID 0x002`, `LOCAL_PEEREPID 0x003`, `LOCAL_PEERUUID`, `LOCAL_PEEREUUID`, `LOCAL_PEERTOKEN 0x006` and `sun_path[104]`; `sys/ucred.h` `struct xucred` has `cr_version`, `cr_uid`, `cr_ngroups`, `cr_groups[NGROUPS]` and no pid; `sys/socket.h` defines `SO_NOSIGPIPE 0x1022`; `spawn.h` has no `POSIX_SPAWN_SETSID`.
