---
title: "POSIX Child-Process Supervision in Rust: Reaping, Process Groups, PID-Reuse Witnesses, Tree Kills and Detachment"
category: concurrency
languages: [rust, shell]
complexity: advanced
use_cases:
  - "spawning a child from a Rust CLI or daemon and stopping it and its descendants on timeout, cancel or Ctrl-C without leaving zombies or orphans"
  - "signalling a process recorded earlier (a pid in a state file or pidfile) without hitting an unrelated process that reused the pid"
  - "making worker processes die when the supervisor dies, including by SIGKILL, on Linux and on macOS"
  - "choosing between a process group, a cgroup, a PID namespace, a subreaper and a launchd or systemd unit for containing a process tree"
  - "detaching a long-lived child from a short-lived CLI without forking inside a threaded Rust process"
summary: "Primary-source and experiment-backed guidance for supervising child processes from Rust on Linux and macOS: std never kills or reaps a dropped Child and tokio's kill_on_drop kills one pid and reaps only while the runtime runs; a pid stays reserved until its owner reaps it, so a direct parent can signal safely only before wait; process groups (process_group(0), kill(-pgid)) stop a cooperative tree but any descendant can leave with setsid; cgroup.kill is the only reliable tree kill and macOS has no equivalent; a stored pid needs a witness (pidfd on Linux, pid plus start time elsewhere); PR_SET_PDEATHSIG follows the spawning thread and one generation, and the portable alternative is a lifeline pipe; detach by spawning a new session, never by forking a threaded process. Includes a compiled group-terminate routine and fifteen experiments."
provenance: researched
researched: 2026-11-23
sources:
  - https://man7.org/linux/man-pages/man2/wait.2.html
  - https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap04.html
  - https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap03.html
  - https://doc.rust-lang.org/std/process/struct.Child.html
  - https://github.com/rust-lang/rust/blob/master/library/std/src/sys/process/unix/unix.rs
  - https://github.com/rust-lang/rust/pull/112594
  - https://docs.rs/tokio/latest/tokio/process/struct.Command.html
  - https://github.com/tokio-rs/tokio/blob/master/tokio/src/process/unix/orphan.rs
  - https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md
  - https://github.com/tokio-rs/tokio/blob/master/tokio/src/process/unix/pidfd_reaper.rs
  - https://github.com/tokio-rs/tokio/blob/master/tokio/src/process/unix/mod.rs
  - https://docs.rs/tokio/latest/tokio/process/struct.Child.html
  - https://man7.org/linux/man-pages/man2/pidfd_open.2.html
  - https://man7.org/linux/man-pages/man7/signal.7.html
  - https://man7.org/linux/man-pages/man2/setpgid.2.html
  - https://pubs.opengroup.org/onlinepubs/9799919799/functions/setpgid.html
  - https://man7.org/linux/man-pages/man2/setsid.2.html
  - https://man7.org/linux/man-pages/man2/kill.2.html
  - https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap11.html
  - https://nexte.st/docs/design/architecture/signal-handling/
  - https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html
  - https://github.com/rust-lang/rust/issues/105376
  - https://github.com/watchexec/command-group
  - https://docs.rs/process-wrap/latest/process_wrap/
  - https://docs.rs/nix/0.31.3/nix/unistd/index.html
  - https://docs.rs/rustix/latest/rustix/process/index.html
  - https://github.com/torvalds/linux/blob/master/kernel/pid.c
  - https://man7.org/linux/man-pages/man5/proc_sys_kernel.5.html
  - https://github.com/systemd/systemd/blob/main/sysctl.d/50-pid-max.conf
  - https://lkml.iu.edu/hypermail/linux/kernel/2501.2/02922.html
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/proc_internal.h
  - https://lwn.net/Articles/963749/
  - https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html
  - https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html
  - https://github.com/rust-lang/rust/issues/82971
  - https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/proc_info.h
  - https://psutil.io/api/
  - https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html
  - https://keith.github.io/xcode-man-pages/kqueue.2.html
  - https://man7.org/linux/man-pages/man2/PR_SET_CHILD_SUBREAPER.2const.html
  - https://man7.org/linux/man-pages/man2/PR_SET_PDEATHSIG.2const.html
  - https://man7.org/linux/man-pages/man2/fork.2.html
  - https://patchew.org/linux/20250913-fix-prctl-pdeathsig-race-v1-1-44e2eb426fe9@gmail.com
  - https://man7.org/linux/man-pages/man7/pid_namespaces.7.html
  - https://github.com/Yelp/dumb-init
  - https://docs.docker.com/reference/cli/docker/container/run/
  - https://github.com/krallin/tini
  - https://docs.kernel.org/admin-guide/cgroup-v2.html
  - https://lwn.net/Articles/855924/
  - https://www.freedesktop.org/software/systemd/man/latest/systemd.kill.html
  - https://www.freedesktop.org/software/systemd/man/latest/systemd-run.html
  - https://github.com/apple-oss-distributions/xnu/blob/main/libsyscall/wrappers/libproc/libproc.h
  - https://keith.github.io/xcode-man-pages/launchd.plist.5.html
  - https://docs.docker.com/reference/cli/docker/container/stop/
  - https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/
  - https://www.freedesktop.org/software/systemd/man/latest/systemd-system.conf.html
  - https://github.com/rust-lang/cargo/issues/14929
  - https://www.freedesktop.org/software/systemd/man/latest/daemon.html
  - https://docs.rs/nix/0.31.3/nix/unistd/fn.fork.html
  - https://github.com/tokio-rs/tokio/issues/6770
  - https://man7.org/linux/man-pages/man3/posix_spawn.3.html
  - https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/spawn.h
  - https://keith.github.io/xcode-man-pages/daemon.3.html
  - https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html
  - https://www.freedesktop.org/software/systemd/man/latest/logind.conf.html
  - https://man7.org/linux/man-pages/man2/close_range.2.html
  - https://keith.github.io/xcode-man-pages/posix_spawnattr_setflags.3.html
  - https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/on-broken-pipe.html
  - https://doc.rust-lang.org/std/process/struct.ExitStatus.html
  - https://doc.rust-lang.org/std/os/unix/process/trait.ExitStatusExt.html
  - https://man7.org/linux/man-pages/man1/bash.1.html
  - https://github.com/openai/codex/issues/26984
  - https://github.com/anthropics/claude-code/issues/1935
  - https://github.com/anthropics/claude-code/issues/33947
  - https://issues.apache.org/jira/browse/YARN-3678
  - "a2a repository, src/exec.rs at commit 19240f7 (2026-10-23), profiled in docs/development/repos/a2a.md"
  - "docs/development/repos/hq.md (agent.run handler)"
---

# POSIX Child-Process Supervision in Rust

Purpose: the kernel and library facts, and the discipline that follows from them, for a Rust program on Linux or macOS that starts other programs and is responsible for them: a CLI running a build or a test, a harness running coding-agent workers, a daemon running jobs. It answers four questions: who reaps, how to stop a whole tree, how to signal a process you recorded earlier without hitting a stranger, and how to let a child outlive you or make sure it does not. It complements `unix-socket-ipc-singleton-daemons.md` (the daemon's own lifecycle and why a pid is not an identity for a socket peer) and `rust-async-cancellation-graceful-shutdown.md` (what happens inside a process when it is asked to stop).

Citations use `[n]` keyed to the `sources` list in the frontmatter. Quotes are verbatim from the cited page. Claims marked **(verified in this run)** come from the experiments in Appendix A, run on 2026-11-23 on macOS 26.6.2 (Darwin 25.6.0, arm64, rustc 1.99.0) and Linux (kernel 7.2.12, `rust:1` container, rustc 1.100.0, glibc 2.41), both with libc 0.2.191 and tokio 1.54.1.

## 1. Decision ladder

| Need | Linux | macOS |
|---|---|---|
| Stop one child you spawned | Signal through the `Child`, then `wait` (section 2) | Same |
| Stop a cooperative child and its descendants | `process_group(0)`, `kill(-pgid)`, reap the leader last (sections 3, 9) | Same |
| Stop a tree whose members may `setsid` or double-fork | cgroup with `cgroup.kill`, or a PID namespace (section 5) | No containment primitive; a launchd job is the closest (section 5) |
| Signal a process recorded on disk | pidfd if you can hold one; otherwise pid plus start time (section 4) | pid plus start time (section 4) |
| Children die when the supervisor dies | `PR_SET_PDEATHSIG` for one generation, a cgroup for the tree (section 6) | Lifeline pipe, or the child watches the parent with kqueue (section 6) |
| A child outlives the CLI | `systemd-run --user`, else a new process in a new session (section 7) | launchd agent, else a new process in a new session (section 7) |
| Run as PID 1 in a container | `docker run --init`, or reap and handle signals yourself (section 8) | n/a |

## 2. Reaping: who calls wait, and what std and tokio do not do

**A dead child is a zombie until its parent waits.** "A child that terminates, but has not been waited for becomes a "zombie"" [1]. The zombie keeps its pid: "A process ID shall not be reused by the system until the process lifetime ends" [2], and the lifetime ends when the pid "is returned to the system" by a wait [3].

**std does nothing on drop.** "There is no implementation of `Drop` for child processes, so if you do not ensure the `Child` has exited then it will continue to run", and for a child that has exited, "Leaving too many zombies around may exhaust global resources (for example process IDs)" [4]. A dropped `Child` whose process had exited was a zombie (`stat=Z`) for as long as the parent lived, on both systems **(verified in this run)**.

**`Child::kill` is SIGKILL to one pid and does not wait.** "This is equivalent to sending a SIGKILL on Unix platforms" and "If the child has already exited, `Ok(())` is returned" [4]. std will not signal a pid it has already reaped: `send_signal` returns early when the exit status is cached, with a source comment that the pid can be recycled [5]. That has been `Ok(())` since Rust 1.72.0; before PR #112594 it was `Err(InvalidInput)` [6]. Kill after wait returned `Ok(())` and a second `wait` returned the cached status **(verified in this run)**. The guard covers `Child` only: `libc::kill(child.id() as i32, ...)` after `wait`, or any pid copied elsewhere, has no protection.

**`wait` and `try_wait`.** `wait` closes the child's stdin first; `try_wait` reaps without blocking: "If the child has exited then on Unix the process ID is reaped" [4].

**tokio.** "a spawned process will, by default, continue to execute even after the `Child` handle has been dropped" [7]. `kill_on_drop(true)` sends the kill in the destructor but cannot await the exit there, so "The tokio runtime will, on a best-effort basis, attempt to reap and clean up any process which it has spawned" [7]. The orphan queue is drained by the runtime's signal driver, which only starts listening for SIGCHLD when there are orphans [8]. Measured **(verified in this run)**, tokio 1.54.1, current-thread runtime, both systems:

| Situation | Child afterwards |
|---|---|
| `kill_on_drop(true)`, handle dropped, runtime thread blocked for 500 ms | zombie |
| Same, then the runtime ran for 500 ms | reaped |
| No `kill_on_drop`, handle dropped | still running; reaped by the runtime after it exited on its own |
| `kill_on_drop(true)`, handle dropped, runtime dropped at once | zombie until the process exits |

On Linux tokio waits through a pidfd since 1.36.0 ("process: use pidfd on Linux when available") and falls back to the SIGCHLD reaper when `pidfd_open` fails [9][10]. On macOS and other Unix it is SIGCHLD-driven, and the module notes that signals coalesce so every SIGCHLD checks all spawned children [11]. `tokio::process::Child::id()` returns `None` once the child has been polled to completion [12]. tokio's `kill` and `kill_on_drop` target the direct pid only, never a group [11].

**Three ways to break reaping.**

1. *Ignoring SIGCHLD.* With SIGCHLD set to `SIG_IGN` (or `SA_NOCLDWAIT`), children are reaped by the kernel and `wait` fails with `ECHILD` [1]. In that state `Child::wait`, `Command::status` and `Command::output` all returned `Err(ECHILD)` on both systems **(verified in this run)**, and the "my unreaped child's pid is stable" guarantee is gone [13]. The default disposition (listed as "Ign" in signal(7)) is not the same as an explicit `SIG_IGN` and does not auto-reap [14][1].
2. *`waitpid(-1)` or `wait()` in library code.* It collects children that belong to someone else in the process; pidfd_open(2) lists "reaped elsewhere in the program" as a condition that breaks pid stability [13].
3. *One SIGCHLD per child.* "Standard signals do not queue" [14]; a handler must loop over its own children with `WNOHANG`.

**`kill(pid, 0)` is not a liveness test for your own children.** It returned success for a zombie on both systems **(verified in this run)**. Use `try_wait`.

**Exit status.** `ExitStatus` is a wait status: `code()` "will return `None` if the process was terminated by a signal" [70], and `ExitStatusExt::signal()` gives the signal [71]. `128+n` is a shell convention ("128+n if the command is terminated by signal n" [72]), not something the kernel reports: a child killed by SIGKILL gave `code()=None, signal()=Some(9)`, and the same death seen through an intermediate `sh` gave `$?` 137 **(verified in this run)**. Synthesize `128+n` only at a shell-facing boundary.

## 3. Process groups: stopping a cooperative tree

**Semantics.** A process group id is inherited across fork and kept across exec [15]. `kill(-pgid, sig)` signals every member: "If pid is less than -1, then sig is sent to every process in the process group whose ID is -pid" [18]. A pid that is also a live group id is not reused "until the process group lifetime ends" [2], so a group id stays valid while any member lives.

**In Rust.** `CommandExt::process_group(0)` (stable since 1.64.0) makes the child the leader of a new group whose id is its pid; it is "Equivalent to a `setpgid` call in the child process, but may be more efficient" [21] and keeps std on the `posix_spawn` path [5]. tokio's `Command::process_group` has been stable since 1.40.0 [9]. `CommandExt::setsid` is still nightly-only (feature `process_setsid`, tracking issue open since 2022) [22]. A `pre_exec` closure is `unsafe`, runs after fork where "`malloc`, accessing environment variables through `std::env` or acquiring a mutex are not guaranteed to work" [21], and forces std onto fork plus exec instead of `posix_spawn` [5]. The classic parent-and-child double `setpgid` exists so that "it does not matter which succeeds first" [16]; with `process_group` the group is set before the program runs, so no parent-side call is needed.

**Crates.** `command-group` is retired: "The successor of command-group is process-wrap. No further work will be done on command-group" [23]. `process-wrap` provides composable wrappers (`ProcessGroup`, `ProcessSession`, `KillOnDrop`, `ResetSigmask`) for std and tokio [24]. `nix::unistd` and `rustix::process` expose the raw calls, including `rustix`'s `kill_process_group`, `pidfd_open`, `pidfd_send_signal` and `set_parent_process_death_signal` [25][26].

**What a group does not give you.**

- *Membership is voluntary.* A process leaves by "calling `setsid()` or `setpgid()`" [3]. A grandchild that called `setsid()` survived the group kill and was reparented to pid 1, on both systems **(verified in this run)**. Section 10 has a field report of a group kill missing the real server process.
- *The terminal stops talking to the child.* Ctrl-C is delivered "to all processes in the foreground process group" [19]. A child in its own group is not in it, so the supervisor must forward SIGINT, SIGTERM and the stop/continue pair itself. nextest states the consequence: "nextest creates a new process group for each unit of work" and "It is then nextest's responsibility to send the signal to all child process groups" [20].
- *Terminal reads stop the child.* A background group that reads the controlling terminal is "sent a SIGTTIN signal, which suspends it" [15][19]. Give a grouped child `Stdio::null()` or a pipe for stdin; stdin is inherited by default [7].
- *A new session is stronger and stricter.* `setsid` gives a new session and group with no controlling terminal, and "fails if the calling process is already a process group leader" [17]. Use it for detachment (section 7), not just for kill scope.

**Order of operations.** Signal the group while the leader is still unreaped. Until the supervisor waits on the leader, the leader's pid, and therefore the group id, cannot be reused [2][13]; after the wait, the id is protected only while other members remain [2]. Section 9 shows a routine that peeks with `waitid(..., WNOWAIT)` ("Leave the child in a waitable state" [1]), kills the group, and reaps last.

**Grace periods in common supervisors**, for calibration: Docker stop waits "10 seconds for Linux containers" [55]; a Kubernetes Pod's term "defaults to 30 seconds" [56]; systemd's `DefaultTimeoutStopSec=` defaults to 90 s [57]; launchd's `ExitTimeOut` default "is system-defined" [54]. SIGKILL "cannot be caught, blocked, or ignored" [14], and it does not propagate: the children of a SIGKILLed parent kept running with parent pid 1 on both systems **(verified in this run)**, which is the cargo behaviour reported in [58].

## 4. PID reuse: a pid is a name, not a handle

**How fast pids come back.** Linux allocates cyclically and wraps at `pid_max` [27]. The kernel default is 32768 and the 64-bit ceiling is 2^22 [28]; systemd raises it to 4194304 "to make PID collisions less likely" [29] (the test container reported 4194304). Since Linux 6.14 `pid_max` is per PID namespace, so a container can have a much smaller range than its host [30]. On macOS `PID_MAX` is 99999 [31]. Reuse is rare per event and certain over a long enough run: Apache YARN's delayed killer was found killing "other process other than container" because "this PID may be occupied by other process" [76].

**The one case that is safe without a witness.** A parent's own unreaped child: the pid "will not have been recycled" as long as SIGCHLD is not ignored and nothing else in the program reaped it [13]. In a privileged container, with `ns_last_pid` rewound, a reaped child's pid was handed to the next spawn, and an unreaped zombie's pid was skipped **(verified in this run)**.

**Everything else needs a witness.** A pid read from a pidfile, a state file, another process's message or an earlier run of your own CLI can name a different process by the time you signal it. systemd's advice is "PID files should be avoided in modern projects" [33].

*Linux: pidfd.* A pidfd is "a Linux-specific way of referring to processes that avoids the race conditions inherent in Unix process ID numbers" [32]. Timeline: `pidfd_send_signal` 5.1, `CLONE_PIDFD` 5.2, `pidfd_open` 5.3, `waitid(P_PIDFD)` 5.4, `PIDFD_NONBLOCK` 5.10, group signalling through a pidfd 6.9 [13][34][1]. A pidfd polls readable when the process exits and reports hangup once it is reaped [13]; signalling a reaped target fails with `ESRCH`: "The target process does not exist (i.e., it has terminated and been waited on)" [34]. All of that held on kernel 7.2 **(verified in this run)**: `poll` returned 0 before exit and `POLLIN` after, signal 0 succeeded on the zombie, and after `wait` the same descriptor returned `ESRCH`. Limits: a pidfd must be opened while you still know who the pid is (as the parent, before reaping), and it is a descriptor, so it does not survive into a later CLI invocation unless a long-lived process holds it. In Rust the std API (`CommandExt::create_pidfd`, `ChildExt::pidfd`) is still unstable [35] and std does not use pidfds unless asked [5]; use `rustix` or raw syscalls.

*Portable: pid plus start time.* On Linux, `/proc/<pid>/stat` field 22 is "The time the process started after system boot", in clock ticks [36]; the second field is the command name in parentheses and can contain spaces, so parse from the last `)`. On macOS, `proc_pidinfo(PROC_PIDTBSDINFO)` fills a `proc_bsdinfo` with `pbi_start_tvsec` and `pbi_start_tvusec` [37]. psutil uses exactly this identity: "two instances are equal if they have the same PID and creation time", and its signal methods re-check it [38]. The `sysinfo` crate documents a reuse hazard on macOS and FreeBSD for `wait()` ("it will wait for the new process to end") [39], so do not assume a process library checks for you. Section 9 has a compiled `start_time` and `signal_if_same`; a wrong start time left the process untouched and the right one signalled it, on both systems **(verified in this run)**. Two measured differences: for a zombie, Linux still reports a start time while macOS `proc_pidinfo` returns nothing; and the Linux value is relative to boot, so persist a boot identifier with it.

The check and the signal are two system calls, so a window remains; it is as small as the time between them, against the minutes or days a bare stored pid is exposed.

*macOS extras.* kqueue's `EVFILT_PROC` with `NOTE_EXIT` reports the exit of any process the caller can see [40]. It is attached by pid, so it tells you when a process ends but not whether a stored pid is still the same process.

## 5. Killing a whole tree when members do not cooperate

**Linux: cgroup v2.** "Writing "1" to the file causes the cgroup and all descendant cgroups to be killed", and "Killing a cgroup tree will deal with concurrent forks appropriately and is protected against migrations" [49][50]. `cgroup.kill` (Linux 5.14) is SIGKILL only; for a graceful pass, signal the pids in `cgroup.procs` first. A `setsid` escapee placed in a cgroup with its parent shell was listed in `cgroup.procs` under its own session, and one write to `cgroup.kill` emptied the cgroup (`populated 0`) **(verified in this run)**. An unprivileged CLI gets a cgroup from the user's systemd manager: `systemd-run --user --scope` "may be used to create and start a transient .service or .scope unit and run the specified COMMAND in it" [52]. A systemd service gets this by default through `KillMode=control-group`, and "it is not recommended to set `KillMode=` to `process` or even `none`" [51].

**Linux: PID namespace.** When a namespace's init exits, "the kernel terminates all of the processes in the namespace via a SIGKILL" [45]. This is what a container gives you.

**Linux: subreaper.** `PR_SET_CHILD_SUBREAPER` (Linux 3.4) makes orphaned descendants reparent to you: "A subreaper fulfills the role of init(1) for its descendant processes" [41]. With it set, an orphaned grandchild's parent became the supervisor instead of pid 1 and the supervisor's `waitpid` collected it **(verified in this run)**. It tells you about escapees and obliges you to reap them; it does not kill them.

**macOS.** There are no cgroups. libproc declares `proc_listchildpids` and `proc_listpgrppids` [53], which allow an enumerate-then-signal sweep that misses anything already reparented. launchd offers one tree-scoped cleanup: "When a job dies, `launchd` kills any remaining processes with the same process group ID as the job", unless `AbandonProcessGroup` is set [54]; a descendant that changed group escapes that too.

## 6. Making children die with the supervisor

A supervisor that is SIGKILLed, crashes or is OOM-killed runs no cleanup code, so the mechanism has to live in the kernel or in the child.

**Linux: `PR_SET_PDEATHSIG`.** Set in the child between fork and exec. Three documented traps [42][43]:

- It tracks a thread: the signal is sent "upon subsequent termination of the parent thread". A child spawned from a short-lived thread was SIGKILLed as soon as that thread returned, while the process lived on **(verified in this run)**. Spawn from a thread that lives as long as the supervisor.
- It covers one generation and is cleared by privilege changes: "The parent-death signal setting is cleared for the child of a fork(2)", and when executing a set-id or capability-bearing binary.
- It has a start-up race: if the parent is already gone when the call is made, "no parent-death signal is sent". Record the parent's pid before spawning and compare `getppid()` after the `prctl` [44]; do not compare with 1, because a subreaper may be the new parent [41].

With the check in place, SIGKILLing the supervisor removed the child **(verified in this run)**.

```rust
use std::os::unix::process::CommandExt;
let parent = unsafe { libc::getpid() };
unsafe {
    cmd.pre_exec(move || {
        if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        if libc::getppid() != parent { libc::_exit(1); } // parent died before prctl took effect
        Ok(())
    });
}
```

**macOS and portable: the lifeline pipe.** macOS has no parent-death signal. Give the child (or a small wrapper around it) the read end of a pipe whose only write end the supervisor holds; when the supervisor dies for any reason the kernel closes the write end and the reader sees end-of-file. With the child's stdin as the lifeline, SIGKILLing the supervisor ended the child on both systems **(verified in this run)**. It works only if no other process holds the write end: a descriptor opened without `O_CLOEXEC` anywhere in the supervisor is inherited by every child (on Linux `close_range` [67], on macOS `POSIX_SPAWN_CLOEXEC_DEFAULT` [68] bound that), and a child that inherits the write end keeps the lifeline open. A child can also watch its parent's pid with kqueue `NOTE_EXIT` [40].

**For a whole tree** on Linux, put it in a scope or cgroup owned by something that outlives the supervisor's crash (section 5); the death signal alone protects only direct children.

## 7. Detaching: letting a child outlive the CLI

**Do not fork a running Rust program to daemonize it.** "After a fork() in a multithreaded program, the child can safely call only async-signal-safe functions" until it execs [43]; nix adds that "memory allocation may not be async-signal-safe" [60]. A process with a tokio runtime is multithreaded, and the tokio maintainers' advice is to fork before creating the first runtime [61]. The classic recipe (fork, `setsid`, fork again so "the daemon can never re-acquire a terminal again", stdio to `/dev/null`, `chdir("/")`) [59] is safe only at the very start of `main`.

**Spawn a fresh process in a new session instead.** Re-execute your own binary with a hidden subcommand: stdin from `/dev/null`, stdout and stderr to a log file, `setsid` in `pre_exec` (async-signal-safe) or `process-wrap`'s `ProcessSession` [24]. glibc 2.26 has `POSIX_SPAWN_SETSID` [62] and the macOS SDK defines the flag too [63], but std uses it only on Linux glibc [5]. The CLI then exits without waiting; the child is reparented to init or the nearest subreaper [1][41], which reaps it.

**What can still kill it.**

- SIGHUP: when a session leader with a controlling terminal exits, "a SIGHUP signal is sent to each process in the foreground process group" [19]. A new session has no controlling terminal [17], which is why `setsid` rather than a new process group is the detachment primitive.
- systemd-logind: with `KillUserProcesses=` on, "the scope unit corresponding to the session and all processes inside that scope will be terminated" at logout, which "will break tools like screen(1) and tmux(1), unless they are moved out of the session scope" [66]. `setsid` does not leave the scope; `systemd-run --user` does [52].
- launchd: a job must not detach at all. "You must not daemonize your process. This includes calling the `daemon` function, calling `fork` followed by `exec`, or calling `fork` followed by `exit`. If you do, `launchd` thinks your process has died" [65]. On macOS `daemon(3)` itself "is discouraged in favor of using launchd(8)" [64].

**Prefer the service manager when one is there.** Under systemd "none of the initialization steps recommended for SysV daemons need to be implemented" [59] and `Type=forking` is discouraged [33]. A unit or launchd agent also gives the tree cleanup of section 5 and restart policy for free.

**Signal state crosses exec.** "the dispositions of handled signals are reset to the default; the dispositions of ignored signals are left unchanged" and "the signal mask is preserved across execve(2)" [14]. std resets SIGPIPE to default in children [69] and otherwise inherits the mask rather than resetting it [5]; a supervisor that blocks signals on its spawning thread hands that mask to every child (`process-wrap`'s `ResetSigmask` exists for this [24]).

## 8. PID 1 in a container

PID 1 inherits every orphan in its namespace and must reap them: "The init system is thus responsible for `wait()`-ing on orphaned zombie processes" [46]. It also gets no default signal actions: "Only signals for which the "init" process has established a signal handler can be sent" to it from inside the namespace [45], so a Rust binary run as PID 1 without a SIGTERM handler ignores `docker stop` until the 10-second SIGKILL [55]. `docker run --init` puts tini in front: "Run an init inside the container that forwards signals and reaps processes" [47]; tini can also act as a subreaper (`-s`) and signal the child's whole group (`-g`) [48].

## 9. Reference implementation

Compiled with rustc 1.99.0 (macOS) and 1.100.0 (Linux) against libc 0.2.191, and exercised by experiments 2 and 7 in Appendix A. Blocking std code; in tokio the same shape applies with `tokio::process::Command::process_group(0)`, `nix` or `rustix` for the signals, and the runtime's timer instead of `sleep`.

```rust
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn kill(pid: i32, sig: i32) -> io::Result<()> {
    if unsafe { libc::kill(pid, sig) } == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

/// Spawn `cmd` as the leader of a new process group (pgid == child pid).
fn spawn_group(cmd: &mut Command) -> io::Result<Child> {
    cmd.process_group(0).spawn()
}

/// Has the direct child exited? Peeks with WNOWAIT, so the child stays a zombie
/// and its pid (which is also the group id) stays reserved.
fn exited_unreaped(pid: i32) -> io::Result<bool> {
    let mut si: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let flags = libc::WEXITED | libc::WNOHANG | libc::WNOWAIT;
    if unsafe { libc::waitid(libc::P_PID, pid as libc::id_t, &mut si, flags) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(si.si_signo == libc::SIGCHLD) // left zeroed when no child has changed state
}

/// Stop a child started with `spawn_group`: SIGTERM the group, give the leader up to
/// `grace` to exit, SIGKILL whatever is left in the group, then reap the leader.
/// The leader is reaped last on purpose: until then the group id cannot be reused.
fn terminate_group(child: &mut Child, grace: Duration) -> io::Result<ExitStatus> {
    if let Some(status) = child.try_wait()? {
        return Ok(status); // already reaped by us earlier: the group id is no longer ours to signal
    }
    let pgid = child.id() as i32;
    let _ = kill(-pgid, libc::SIGTERM);
    let deadline = Instant::now() + grace;
    while !exited_unreaped(pgid)? && Instant::now() < deadline {
        sleep(Duration::from_millis(20));
    }
    // Leader exited (grandchildren may remain) or the grace period ran out.
    let _ = kill(-pgid, libc::SIGKILL);
    child.wait()
}

/// Start time of `pid` in the kernel's own units, or None if no such process.
/// Together with the pid this identifies one process instance across pid reuse.
#[cfg(target_os = "linux")]
fn start_time(pid: i32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // comm may contain spaces and parentheses: split after the LAST ')'.
    let rest = &stat[stat.rfind(')')? + 2..];
    rest.split(' ').nth(19)?.parse().ok() // field 22 overall: starttime, clock ticks since boot
}
#[cfg(target_os = "macos")]
fn start_time(pid: i32) -> Option<u64> {
    let mut bi: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    let n = unsafe {
        libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, &mut bi as *mut _ as *mut libc::c_void, size)
    };
    if n != size { return None; }
    Some(bi.pbi_start_tvsec * 1_000_000 + bi.pbi_start_tvusec)
}

/// Signal a process recorded earlier as (pid, start_time) only if it is still that process.
fn signal_if_same(pid: i32, recorded_start: u64, sig: i32) -> io::Result<bool> {
    match start_time(pid) {
        Some(now) if now == recorded_start => kill(pid, sig).map(|_| true),
        _ => Ok(false), // gone, or the pid now names another process
    }
}
```

Measured with a shell and two `sleep` children in the group: a cooperative group was gone about 100 ms after the call (one 20 ms poll plus process teardown); a shell that traps SIGTERM was killed at the end of a 500 ms grace period; no group members remained in either case, on both systems **(verified in this run)**.

## 10. Failures seen in the field

| Report | What went wrong | Section |
|---|---|---|
| openai/codex #26984 (open, 2026) | MCP stdio servers started with `process_group(0)` behind an `npx` wrapper; the group kill misses the real server, and orphans holding pipe write ends make the parent's descriptors "grow **monotonically** until EMFILE" (reporter's analysis, no maintainer confirmation) [73] | 3, 5 |
| anthropics/claude-code #1935 (open since 2025-06) | "MCP servers not properly terminated when Claude Code exits, causing orphaned processes" [74]; a duplicate counted 107 orphaned node processes and noted "macOS lacks `prctl(PR_SET_PDEATHSIG)`" [75] | 6 |
| rust-lang/cargo #14929 (open) | SIGKILL of `cargo test` leaves a hung test binary running; cargo does no signal handling of its own there [58] | 3 |
| Apache YARN-3678 | A delayed kill hit an unrelated process after the container's pid was reused [76] | 4 |
| tokio #6770 (closed, no root cause) | `wait()` never resolved and the child stayed a zombie on macOS; one reproduction forked after creating the runtime [61] | 2, 7 |

## 11. Worked examples from our repositories

- **a2a** (`src/exec.rs`): a worker is a detached session (the spawner `setsid`s in `pre_exec`) whose liveness is a `flock` held by a shim process, not a polled pid. Every kill first compares a recorded start-time witness and refuses on mismatch, then sends SIGTERM to the group, polls for two seconds and sends SIGKILL [77]. The witness is the output of `ps -o lstart=` [77].
- **hq** (`agent.run`): spawns the agent command and SIGKILLs its process group at `timeout_secs` (default 600) [78].

## 12. Checklist

1. Every spawned child has one owner that waits on it; no code path drops a `Child` without `wait`, and nothing sets SIGCHLD to `SIG_IGN` or calls `waitpid(-1)`.
2. With tokio, `kill_on_drop` is treated as a last resort: the normal path kills and awaits `wait()`, and the runtime outlives its children.
3. A child that may have descendants is spawned with `process_group(0)` and stopped with `kill(-pgid)`: SIGTERM, a grace period, SIGKILL, and the leader reaped last.
4. A grouped child has stdin set to null or a pipe, and the supervisor forwards SIGINT and SIGTERM to its groups.
5. No bare pid is signalled unless it is the caller's own unreaped child; anything stored carries a start time (and a boot id on Linux) or is held as a pidfd.
6. Trees that may contain `setsid` or double-fork run in a cgroup, a container or a service-manager unit on Linux; on macOS the residual escape is written down.
7. Workers that must not outlive the supervisor have a kernel-level tie: `PR_SET_PDEATHSIG` with the `getppid` check from a long-lived thread, or a lifeline pipe.
8. Detachment is a fresh process in a new session, or a systemd or launchd unit; never a fork of a threaded process.
9. As PID 1 in a container, either `--init` is used or the binary reaps orphans and handles SIGTERM.
10. Exit handling reads `code()` then `signal()`.

## 13. Common misconceptions

| Belief | What the sources and experiments show |
|---|---|
| Dropping a `Child` cleans it up | std neither kills nor waits [4]; tokio kills only with `kill_on_drop` and reaps only while the runtime runs [7][8] |
| `Child::kill` after `wait` can hit a recycled pid | std returns `Ok(())` without signalling since 1.72 [5][6]; the hazard is raw `kill` on a copied pid |
| std uses pidfds on modern Linux | Only with the unstable `create_pidfd` [5][35]; tokio does since 1.36.0 [9] |
| tokio uses kqueue process events on macOS | It uses SIGCHLD [11] |
| `process_group` forces the slow fork path | It stays on `posix_spawn`; `pre_exec` is what forces fork [5] |
| Killing the group kills the tree | Any member can leave with `setsid` or `setpgid` [3]; verified |
| The group id can be recycled under you at any time | It is reserved while any member lives [2]; the exposed case is an empty group after the leader was reaped |
| `PR_SET_PDEATHSIG` means "die with the parent process" | Parent thread, one generation, cleared on set-id exec, with a start-up race [42][43] |
| `getppid() == 1` detects an orphan | Not under a subreaper [41]; compare with the recorded parent pid [44] |
| `command-group` is the crate for this | Retired in favour of `process-wrap` [23][24] |
| Background output to the terminal stops a job | Reads do; writes only when `TOSTOP` is set [19] |
| Exit code 137 comes from the kernel | The kernel reports "killed by signal 9"; 128+n is added by shells [72] |

## 14. Synthesis (inferred)

These are our own conclusions from the sources and experiments above, not claims any single source makes.

- **Supervision is three separate problems that share vocabulary.** Reaping is about your direct children and is solved by ownership. Scope (what dies when you say stop) is solved by a kernel container: group, session, cgroup, namespace. Identity (is this still the process I meant) is solved by a witness. Most leaks in section 10 come from using a tool for one problem as if it solved another: a group for identity, a pid for scope, `kill_on_drop` for reaping.
- **Prefer liveness that the kernel revokes over liveness you poll.** A lock, pipe or pidfd held by the process disappears when the process does, in any way it dies; a pid has to be checked. a2a's `flock` liveness plus start-time witness and the lifeline pipe are the same idea applied in two directions (supervisor watching worker, worker watching supervisor).
- **For coding-agent harnesses, close stdin first.** Stdio servers that exit on end-of-file need no signal at all, and closing the pipe also removes the descriptor build-up described in [73]. The order we would use is: close stdin, short wait, SIGTERM to the group, grace, SIGKILL to the group, reap.
- **`ps -o lstart=` is a weaker witness than the kernel values.** It has one-second resolution and costs a process spawn per check; a pid recycled within the same second would pass. The microsecond value from `proc_pidinfo` and the tick count from `/proc` are cheaper and finer. Moving a2a to them is a small change, and the difference only matters under very fast pid turnover (for example a small per-namespace `pid_max` [30]).
- **On macOS, accept that containment is best-effort and design the worker to cooperate.** With no cgroup equivalent, the reliable levers are on the child's side: a lifeline it watches, and a session it does not leave. For third-party children that will not cooperate, a launchd job is the only scope the system cleans up for you.
- **The spawn-thread rule for `PR_SET_PDEATHSIG` rules out `spawn_blocking`.** tokio's blocking-pool threads are reclaimed when idle, so a child spawned from one would receive the death signal when the pool shrinks. This follows from the thread semantics in [42] and our experiment 5; we did not test it against tokio's pool directly.

## Appendix A: experiments run for this document (2026-11-23)

One Rust binary (std plus libc 0.2.191 and tokio 1.54.1), run natively on macOS 26.6.2 (rustc 1.99.0) and in a `rust:1` container on Linux 7.2.12 (rustc 1.100.0, glibc 2.41, `--init`); experiments 8 and 14 in a `--privileged` container. "Both" means the same result on macOS and Linux.

1. `Child` of `true` dropped without `wait`: `stat=Z` after 300 ms; `kill(pid, 0)` returns 0 on the zombie; after a manual `waitpid` the pid is gone and `kill(pid, 0)` gives `ESRCH` (both). `start_time(zombie)`: `None` on macOS, a value on Linux.
2. `terminate_group` (section 9) on `sh -c 'sleep 300 & sleep 300 & wait'` spawned with `process_group(0)`: three members before; leader status signal 15; no members after, in 78 to 120 ms (both). With `trap '' TERM` in the shell and a 500 ms grace: leader status signal 9 after about 540 to 590 ms; no members after (both). With a grandchild that calls `setsid()`: it has its own group and session before, and after the group kill it is still running with parent pid 1 (both).
3. `wait`, then `kill`: `Ok(())`; second `wait` and `try_wait`: the cached status. `kill` on an exited but unreaped child: `Ok(())` (both).
4. Intermediate parent SIGKILLed: its `sleep` child keeps running, parent pid 1 (both).
5. Linux: `pre_exec` with `PR_SET_PDEATHSIG, SIGKILL` and the `getppid` check; intermediate parent SIGKILLed: child gone. Spawned from a thread that then exits while the process lives: `try_wait` shows the child killed by signal 9.
6. Lifeline: child `sh -c 'cat >/dev/null; exit 0'` with piped stdin held by the intermediate parent; parent SIGKILLed: child gone within 500 ms (both).
7. `start_time` stable across two reads; `signal_if_same` with a wrong start: `false`, process still running; with the right start: `true`, child dies of signal 15; after reap: `None` and `false` (both).
8. Linux, privileged: child reaped, `ns_last_pid` set to pid-1, next spawn gets the same pid. Left as a zombie, `ns_last_pid` set again: next spawn gets pid+1.
9. Linux: `pidfd_open` on a running child; `poll` 0; `pidfd_send_signal(SIGTERM)` 0; `poll` 1 with `POLLIN`; signal 0 on the zombie succeeds; after `wait`, `pidfd_send_signal` fails with `ESRCH`.
10. `sh -c 'kill -9 $$'`: `code()=None`, `signal()=Some(9)`, displayed as `signal: 9 (SIGKILL)`; seen through an outer shell, `$?` is 137; `exit 3`: `code()=Some(3)` (both).
11. SIGCHLD set to `SIG_IGN`: `Child::wait`, `Command::status` and `Command::output` all return `Err` with `ECHILD` (errno 10) (both).
12. tokio current-thread runtime: the four rows of the table in section 2 (both).
13. Linux: `PR_SET_CHILD_SUBREAPER` set; intermediate parent SIGKILLed: the grandchild's parent pid becomes the supervisor's, and the supervisor's `waitpid` on it succeeds (without the flag, experiment 4, `waitpid` fails with `ECHILD`).
14. Linux, privileged: shell moved into a new cgroup, with a `setsid` grandchild and a `sleep`; `cgroup.procs` lists all three, `populated 1`; `echo 1 > cgroup.kill`; `cgroup.procs` empty, `populated 0`, `rmdir` succeeds.
15. Sections 6 and 9 code: `cargo build --release` clean on both toolchains.

Not tested: terminal behaviour (Ctrl-C delivery, SIGTTIN), systemd units and `systemd-run`, launchd jobs, logind's `KillUserProcesses`, tokio's blocking pool with `PR_SET_PDEATHSIG`, and `POSIX_SPAWN_SETSID` at run time on macOS. Those statements rest on the cited documentation.
