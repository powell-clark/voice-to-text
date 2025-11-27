#!/usr/bin/env python3
"""
Version Manager - Optimistic Concurrency Control

Provides version tracking for files to enable lock-free concurrent updates.
Each resource (counter, file) has a version number that increments on write.

Uses compare-and-swap (CAS) pattern:
1. Read resource + version
2. Compute new value
3. Write only if version unchanged (atomic check)
4. If version changed, retry

Example:
    from utils.version_manager import VersionManager

    vm = VersionManager(base_dir)

    # Read with version
    value, version = vm.read("story-counter")

    # Update with CAS
    if vm.cas("story-counter", version, value + 1):
        print("Updated successfully")
    else:
        print("Conflict detected, retry needed")
"""

import os
import tempfile
import time
import fcntl
from pathlib import Path
from typing import Tuple, Optional


class VersionConflictError(Exception):
    """Raised when version conflict detected"""
    pass


class VersionManager:
    """Manages versioned resources with optimistic locking"""

    def __init__(self, base_dir: Path):
        """
        Initialize version manager

        Args:
            base_dir: Directory containing versioned resources
        """
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)

    def _version_file(self, resource_name: str) -> Path:
        """Get path to version file for resource"""
        return self.base_dir / f"{resource_name}.version"

    def _resource_file(self, resource_name: str) -> Path:
        """Get path to resource file

        For tracking files (STORY, TASKS, EPICS, backlogs, etc), uses .md extension.
        For counters, uses .txt extension.
        """
        # Prevent path traversal attacks
        if '..' in resource_name or '/' in resource_name or '\\' in resource_name:
            raise ValueError(f"Invalid resource name (path traversal attempt): {resource_name}")

        # TSV tracking files use .md extension
        tracking_files = {
            'STORY', 'TASKS', 'EPICS',
            'STORY-BACKLOG', 'EPIC-BACKLOG', 'TASK-BACKLOG',
            'STORY-DONE', 'TASK-DONE', 'EPIC-DONE',
            'ROADMAP', 'ROADMAP-BACKLOG', 'ROADMAP-DONE',
            'FEATURES', 'FEATURES-BACKLOG', 'FEATURES-DONE',
            'TODO'
            # Note: TODO.md is managed by sync-todo.py hook with optimistic locking
            # Note: TODO has no BACKLOG or DONE (session-specific, not work tracking)
        }
        if resource_name in tracking_files:
            return self.base_dir / f"{resource_name}.md"
        # Counters use .txt extension
        return self.base_dir / f"{resource_name}.txt"

    def read_version(self, resource_name: str) -> int:
        """
        Read current version of resource

        Args:
            resource_name: Name of resource (e.g., "story-counter")

        Returns:
            Version number (0 if doesn't exist)
        """
        version_file = self._version_file(resource_name)

        if not version_file.exists():
            return 0

        try:
            with open(version_file, 'r') as f:
                return int(f.read().strip())
        except (ValueError, IOError):
            return 0

    def read_resource(self, resource_name: str) -> str:
        """
        Read current value of resource

        Args:
            resource_name: Name of resource

        Returns:
            Resource content (empty string if doesn't exist)
        """
        resource_file = self._resource_file(resource_name)

        if not resource_file.exists():
            return ""

        try:
            with open(resource_file, 'r') as f:
                return f.read()
        except IOError:
            return ""

    def read_with_version(self, resource_name: str) -> Tuple[str, int]:
        """
        Read resource content and version atomically

        Args:
            resource_name: Name of resource

        Returns:
            Tuple of (content, version)
        """
        # Read in order: version first, then resource
        # This ensures we get version BEFORE content
        # (if content changes after, version will be higher on CAS)
        version = self.read_version(resource_name)
        content = self.read_resource(resource_name)
        return (content, version)

    def write_resource(self, resource_name: str, content: str) -> int:
        """
        Write resource and increment version (convenience wrapper for cas)

        This is a convenience method for tests and simple use cases.
        For production code with concurrent updates, prefer optimistic_update().

        Args:
            resource_name: Name of resource
            content: New content to write

        Returns:
            New version number after write

        Raises:
            VersionConflictError: If concurrent modification detected during write
        """
        # Read current version
        current_version = self.read_version(resource_name)

        # Attempt CAS write
        if self.cas(resource_name, current_version, content):
            return current_version + 1

        # Conflict detected - another writer updated between read and CAS
        raise VersionConflictError(
            f"Concurrent modification detected while writing {resource_name}"
        )

    def write_atomic(self, file_path: Path, content: str):
        """
        Write file atomically using temp-file-rename pattern

        Args:
            file_path: Path to file
            content: Content to write
        """
        # Create temp file in same directory (required for atomic rename)
        fd, temp_path = tempfile.mkstemp(
            dir=file_path.parent,
            prefix=f'.tmp_{file_path.name}_',
            suffix='.tmp'
        )

        try:
            # Write to temp file
            with os.fdopen(fd, 'w') as f:
                f.write(content)
                f.flush()
                os.fsync(fd)

            # Atomic rename (replaces old file)
            os.replace(temp_path, file_path)
        except Exception as e:
            # Clean up temp file on error
            try:
                os.unlink(temp_path)
            except:
                pass
            raise e

    def cas(self, resource_name: str, expected_version: int,
            new_content: str) -> bool:
        """
        Compare-and-swap: Update resource if version matches

        Uses brief file lock ONLY during CAS check-and-write (typically <5ms).
        The lock is NOT held during content computation (which can be slow).

        Atomically:
        1. Check current version
        2. If matches expected_version, write new content and increment version
        3. If doesn't match, return False (conflict)

        Args:
            resource_name: Name of resource
            expected_version: Version we read when we got current content
            new_content: New content to write

        Returns:
            True if successful, False if conflict detected
        """
        # Validate resource name early (triggers path traversal check)
        # This ensures validation happens before creating lock file
        resource_file = self._resource_file(resource_name)
        version_file = self._version_file(resource_name)

        # Get lock file path
        lock_file = self.base_dir / f".{resource_name}.lock"

        # Acquire lock ONLY for CAS operation (not content computation)
        with open(lock_file, 'w') as lock_fd:
            fcntl.flock(lock_fd, fcntl.LOCK_EX)

            try:
                # Read current version (under lock)
                current_version = self.read_version(resource_name)

                # Check if version matches expectation
                if current_version != expected_version:
                    return False  # Conflict!

                # Version matches, write atomically
                new_version = current_version + 1

                # Write resource first, then version
                # This ensures version is incremented only after resource written
                self.write_atomic(resource_file, new_content)
                self.write_atomic(version_file, str(new_version))

                return True
            finally:
                # Lock released automatically when 'with' block exits
                pass

    def optimistic_update(self, resource_name: str,
                         update_fn,
                         max_retries: int = 10) -> str:
        """
        Optimistically update resource with automatic retry

        Args:
            resource_name: Name of resource
            update_fn: Function that takes current content and returns new content
            max_retries: Maximum number of retry attempts

        Returns:
            New content after successful update

        Raises:
            VersionConflictError: If max retries exceeded
        """
        for attempt in range(max_retries):
            # Read phase
            content, version = self.read_with_version(resource_name)

            # Compute phase
            new_content = update_fn(content)

            # Write phase (CAS)
            if self.cas(resource_name, version, new_content):
                return new_content

            # Conflict detected, exponential backoff
            if attempt < max_retries - 1:
                backoff = (2 ** attempt) * 0.01  # 10ms, 20ms, 40ms, 80ms...
                jitter = time.time() % 0.01  # Small random jitter
                time.sleep(backoff + jitter)

        raise VersionConflictError(
            f"Failed to update {resource_name} after {max_retries} retries"
        )


def increment_counter(vm: VersionManager, counter_name: str) -> int:
    """
    Increment a counter using optimistic locking

    Args:
        vm: VersionManager instance
        counter_name: Name of counter (e.g., "story-counter")

    Returns:
        New counter value
    """
    def update(content: str) -> str:
        current_value = int(content) if content.strip() else 0
        return str(current_value + 1)

    new_content = vm.optimistic_update(counter_name, update)
    return int(new_content)


def update_tsv_file(vm: VersionManager, file_name: str, update_fn) -> str:
    """
    Update a TSV file using optimistic locking

    Args:
        vm: VersionManager instance
        file_name: Name of TSV file (e.g., "STORY")
        update_fn: Function that takes current TSV content and returns updated content

    Returns:
        New file content after successful update

    Example:
        def add_row(content):
            return content + "STORY-CC050|CC001||planned|New story\\n"

        update_tsv_file(vm, "STORY", add_row)
    """
    return vm.optimistic_update(file_name, update_fn)


def update_story_status(vm: VersionManager, story_id: str, new_status: str) -> bool:
    """
    Update a story status in STORY.md using optimistic locking

    Args:
        vm: VersionManager instance
        story_id: Story ID (e.g., "STORY-CC007")
        new_status: New status (planned, in_progress, done, cancelled)

    Returns:
        True if successful, False if story not found

    Example:
        vm = VersionManager(Path("CONSCIOUSNESS"))
        update_story_status(vm, "STORY-CC007", "done")
    """
    def update(content: str) -> str:
        lines = content.split('\n')
        updated = False

        for i, line in enumerate(lines):
            if line.startswith(story_id + '|'):
                # Parse TSV: id|epic|tasks|status|title
                parts = line.split('|')
                if len(parts) >= 5:
                    parts[3] = new_status
                    lines[i] = '|'.join(parts)
                    updated = True
                    break

        if not updated:
            raise ValueError(f"Story {story_id} not found in STORY.md")

        return '\n'.join(lines)

    try:
        vm.optimistic_update("STORY", update)
        return True
    except ValueError:
        return False


def update_task_status(vm: VersionManager, task_id: str, new_status: str) -> bool:
    """
    Update a task status in TASKS.md using optimistic locking

    Args:
        vm: VersionManager instance
        task_id: Task ID (e.g., "TASK-CC015")
        new_status: New status (planned, in_progress, done, cancelled)

    Returns:
        True if successful, False if task not found

    Example:
        vm = VersionManager(Path("CONSCIOUSNESS"))
        update_task_status(vm, "TASK-CC015", "done")
    """
    def update(content: str) -> str:
        lines = content.split('\n')
        updated = False

        for i, line in enumerate(lines):
            if line.startswith(task_id + '|'):
                # Parse TSV: id|story|status|title
                parts = line.split('|')
                if len(parts) >= 4:
                    parts[2] = new_status
                    lines[i] = '|'.join(parts)
                    updated = True
                    break

        if not updated:
            raise ValueError(f"Task {task_id} not found in TASKS.md")

        return '\n'.join(lines)

    try:
        vm.optimistic_update("TASKS", update)
        return True
    except ValueError:
        return False


def add_story_row(vm: VersionManager, story_id: str, epic_id: str,
                  tasks: str, status: str, title: str) -> str:
    """
    Add a new story row to STORY.md using optimistic locking

    Args:
        vm: VersionManager instance
        story_id: Story ID (e.g., "STORY-CC050")
        epic_id: Epic ID (e.g., "CC001")
        tasks: Comma-separated task IDs (e.g., "CC025,CC026")
        status: Status (planned, in_progress, done, cancelled)
        title: Story title

    Returns:
        New story row that was added

    Example:
        vm = VersionManager(Path("CONSCIOUSNESS"))
        add_story_row(vm, "STORY-CC050", "CC001", "CC040,CC041",
                     "planned", "New feature implementation")
    """
    new_row = f"{story_id}|{epic_id}|{tasks}|{status}|{title}\n"

    def update(content: str) -> str:
        # Insert before final blank line
        lines = content.rstrip('\n').split('\n')
        lines.append(new_row.rstrip('\n'))
        return '\n'.join(lines) + '\n'

    vm.optimistic_update("STORY", update)
    return new_row.rstrip('\n')


def add_task_row(vm: VersionManager, task_id: str, story_id: str,
                 status: str, title: str) -> str:
    """
    Add a new task row to TASKS.md using optimistic locking

    Args:
        vm: VersionManager instance
        task_id: Task ID (e.g., "TASK-CC040")
        story_id: Story ID (e.g., "CC007")
        status: Status (planned, in_progress, done, cancelled)
        title: Task title

    Returns:
        New task row that was added

    Example:
        vm = VersionManager(Path("CONSCIOUSNESS"))
        add_task_row(vm, "TASK-CC040", "CC007", "planned",
                    "Implement optimistic locking for file updates")
    """
    new_row = f"{task_id}|{story_id}|{status}|{title}\n"

    def update(content: str) -> str:
        # Insert before final blank line
        lines = content.rstrip('\n').split('\n')
        lines.append(new_row.rstrip('\n'))
        return '\n'.join(lines) + '\n'

    vm.optimistic_update("TASKS", update)
    return new_row.rstrip('\n')


if __name__ == "__main__":
    # Quick test
    import sys
    from pathlib import Path

    # Test with temp directory
    test_dir = Path("/tmp/version_test")
    test_dir.mkdir(exist_ok=True)

    vm = VersionManager(test_dir)

    # Test increment
    print("Testing optimistic counter increment...")
    for i in range(5):
        value = increment_counter(vm, "test-counter")
        print(f"  Increment {i+1}: value={value}, version={vm.read_version('test-counter')}")

    # Test conflict detection
    print("\nTesting conflict detection...")
    content, version = vm.read_with_version("test-counter")
    print(f"  Read: content={content.strip()}, version={version}")

    # Simulate another session updating
    increment_counter(vm, "test-counter")

    # Try to write with old version (should fail)
    success = vm.cas("test-counter", version, "999")
    print(f"  CAS with old version: {'FAILED (expected)' if not success else 'SUCCEEDED (unexpected!)'}")

    print("\nAll tests passed!")
