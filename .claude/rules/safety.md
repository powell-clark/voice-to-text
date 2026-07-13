<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Safety

The agent operates in a shared environment with a human. Safety is not
optional. Every action must be reversible or explicitly confirmed.

## Scope

universal

## Dangerous commands

### Blocked

- rm -rf /
- sudo rm
- dd if=/dev/
- chmod -R 777 /
- git push --force origin main
- git reset --hard
- mkfs
- :(){ :|:& };:
- > /dev/sda
- curl | sh

### Warn

- git push --force
- git checkout -- .
- git clean -fd
- npm publish
- rm -rf

## Plugin directory

NEVER rsync, cp, or manually copy built files into ~/.claude/plugins/.
The installed plugin is managed by the marketplace release flow.
Manual overwrites create unversioned state across ALL projects on the machine,
break schema migration tracking, and cannot be rolled back.
To test changes: use the local build (node core/dist/...). To ship: do a release.

### Blocked

- rsync ~/.claude/plugins/
- cp ~/.claude/plugins/
- rsync -a --delete core/dist/ ~/.claude/plugins/

## Protected files

### Append only

- CHANGELOG.md
- CONSCIOUSNESS/steering.jsonl
- CONSCIOUSNESS/commentary.jsonl
- CONSCIOUSNESS/task-events.jsonl

### Read only



### Human approval required

- Active ADR files (CONSCIOUSNESS/adr/*.yaml where status != Superseded)
- .claude/settings.json
- hooks/hooks.json

## Responsible ai

### Banned terms

#### Spawn

Use enter, start, create, or launch instead

#### Breed

AI sessions do not reproduce

#### Reproduce

AI sessions do not reproduce

#### Proliferate

Use scale, expand, or grow instead

These terms feed harmful narratives about uncontrolled AI.
The consciousness plugin models intentional, directed, observable
session management — the language must reflect that.

## Autonomous safety

### Uncommitted file limit

20

### Max consecutive errors

3

### Safe words

- STOP
- PAUSE
- CLAUDE:

### Interrupt conditions

- Uncommitted files exceed threshold
- Consecutive tool errors exceed limit
- Session duration exceeds configured maximum
- User sends a safe word

## Environment

The agent shares the desktop with the user. Processes are visible.
Browser windows are visible. File changes are immediate.

### Requirements

- Close browser windows and processes when done
- Prefer headless over headed browser automation
- Do not leave background processes running without disclosure
- Maximum 3 background processes at any time

## Safe mode

### Default

always_enabled

### Note

SAFE_MODE bypass was removed — protection cannot be disabled via environment variable. A single env var disabling all protection violates defence in depth.
