#!/usr/bin/env python3
"""
Unified schema loader for CONSCIOUSNESS validation.

Single source of truth: precepts/schema.json
Project config: CONSCIOUSNESS/config.json

Usage:
    from utils.schema import load_schema, load_config, get_file_schema
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Optional, Any


def find_project_root() -> Path:
    """Find project root by looking for CONSCIOUSNESS directory."""
    current = Path.cwd()
    for parent in [current] + list(current.parents):
        if (parent / "CONSCIOUSNESS").is_dir():
            return parent
        if (parent / "precepts" / "schema.json").exists():
            return parent
    return current


def load_schema(schema_path: Optional[Path] = None) -> Dict[str, Any]:
    """Load the canonical schema from precepts/schema.json."""
    if schema_path is None:
        root = find_project_root()
        schema_path = root / "precepts" / "schema.json"

    if not schema_path.exists():
        # Try home directory
        home_schema = Path.home() / ".claude" / "precepts" / "schema.json"
        if home_schema.exists():
            schema_path = home_schema
        else:
            raise FileNotFoundError(f"Schema not found: {schema_path}")

    with open(schema_path) as f:
        return json.load(f)


def load_config(config_path: Optional[Path] = None) -> Dict[str, Any]:
    """Load project-specific config from CONSCIOUSNESS/config.json."""
    if config_path is None:
        root = find_project_root()
        config_path = root / "CONSCIOUSNESS" / "config.json"

    if not config_path.exists():
        # Return empty config if not found
        return {"project": {"prefix": "[A-Z]{2,4}"}}

    with open(config_path) as f:
        return json.load(f)


def get_file_schema(filename: str, schema: Optional[Dict] = None) -> Optional[Dict]:
    """Get schema for a specific file."""
    if schema is None:
        schema = load_schema()

    return schema.get("files", {}).get(filename)


def get_header(filename: str, schema: Optional[Dict] = None) -> Optional[str]:
    """Get expected header for a file."""
    file_schema = get_file_schema(filename, schema)
    if file_schema:
        return file_schema.get("header")
    return None


def get_columns(filename: str, schema: Optional[Dict] = None) -> Optional[int]:
    """Get expected column count for a file."""
    file_schema = get_file_schema(filename, schema)
    if file_schema:
        return file_schema.get("columns")
    return None


def get_statuses(status_type: str = "work", schema: Optional[Dict] = None) -> List[str]:
    """Get valid status values."""
    if schema is None:
        schema = load_schema()
    return schema.get("statuses", {}).get(status_type, [])


def get_categories(schema: Optional[Dict] = None) -> List[str]:
    """Get valid category values."""
    if schema is None:
        schema = load_schema()
    return schema.get("categories", [])


def get_priorities(schema: Optional[Dict] = None) -> List[str]:
    """Get valid priority values."""
    if schema is None:
        schema = load_schema()
    return schema.get("priorities", [])


def get_id_pattern(id_type: str, config: Optional[Dict] = None) -> str:
    """
    Get regex pattern for ID validation.

    Args:
        id_type: One of 'EPIC', 'STORY', 'TASK', 'FEAT'
        config: Project config (loads if not provided)

    Returns:
        Regex pattern string
    """
    if config is None:
        config = load_config()

    prefix = config.get("project", {}).get("prefix", "[A-Z]{2,4}")
    return f"^{id_type}-{prefix}\\d{{3}}$"


def get_smart_patterns(schema: Optional[Dict] = None) -> List[str]:
    """Get SMART criteria regex patterns."""
    if schema is None:
        schema = load_schema()
    return schema.get("validation", {}).get("smart_patterns", [])


def get_template(filename: str, schema: Optional[Dict] = None) -> Optional[str]:
    """Get file template content."""
    if schema is None:
        schema = load_schema()
    return schema.get("templates", {}).get(filename)


def get_display_message(filename: str, message_type: str, schema: Optional[Dict] = None) -> str:
    """
    Get display message for pgps output.

    Args:
        filename: File name (e.g., 'STORY.md')
        message_type: One of 'empty', 'missing', 'corrupted'
        schema: Schema dict (loads if not provided)

    Returns:
        Display message string
    """
    file_schema = get_file_schema(filename, schema)
    if file_schema:
        key = f"{message_type}_message"
        return file_schema.get(key, f"[{message_type}]")
    return f"[{message_type}]"


def validate_header(filename: str, actual_header: str, schema: Optional[Dict] = None) -> bool:
    """Check if file header matches expected schema."""
    expected = get_header(filename, schema)
    if expected is None:
        return True  # No header requirement
    return actual_header.strip() == expected


def validate_id(id_value: str, id_type: str, config: Optional[Dict] = None) -> bool:
    """Check if ID matches expected pattern."""
    pattern = get_id_pattern(id_type, config)
    return bool(re.match(pattern, id_value))


def validate_status(status: str, status_type: str = "work", schema: Optional[Dict] = None) -> bool:
    """Check if status is valid."""
    valid = get_statuses(status_type, schema)
    return status in valid


def all_files(schema: Optional[Dict] = None) -> Dict[str, Dict]:
    """Get all file schemas."""
    if schema is None:
        schema = load_schema()
    return schema.get("files", {})


def required_files(schema: Optional[Dict] = None) -> List[str]:
    """Get list of required files."""
    files = all_files(schema)
    return [name for name, info in files.items() if info.get("required", False)]


if __name__ == "__main__":
    # Test the schema loader
    schema = load_schema()
    config = load_config()

    print(f"Schema version: {schema.get('version')}")
    print(f"Project prefix: {config.get('project', {}).get('prefix', 'N/A')}")
    print(f"Required files: {required_files(schema)}")
    print(f"Valid statuses: {get_statuses()}")
    print(f"STORY.md header: {get_header('STORY.md')}")
