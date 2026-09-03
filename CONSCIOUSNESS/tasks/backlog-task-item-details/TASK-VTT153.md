# TASK-VTT153: Verify the running binary matches the installed one

## Context

Three silent deployment failures in one morning, all the same shape: something reports success while the old code keeps running. (1) A .deb shipped a two-week-old prebuilt through four green stages — now gated by TASK-VTT152. (2) apt refuses an unchanged version and exits 0, so the install script printed Installed over a no-op. (3) After a correct dpkg -i, the running process still held the replaced inode — /proc/PID/exe showed '/usr/bin/vtt-linux (deleted)' — and because that process was launched by hand rather than systemd, MainPID was 0, so systemctl restart restarted an empty unit while the singleton lock kept the replacement out. Wanted: a vtt-linux --doctor or a postinst check that compares the running process's inode and version against the on-disk binary and says plainly when they differ, plus a startup log line naming the build. The capture-rate log line was the only thing that distinguished the builds today, and that was luck rather than design.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
