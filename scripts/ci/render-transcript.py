#!/usr/bin/env python3
"""Render a Claude Code execution-output JSON into readable markdown.

Used in CI to write the transcript to $GITHUB_STEP_SUMMARY so the chat shows
inline on the run page. Input: the claude-code-action `execution_file` output
(a JSON array of stream events). Output: markdown on stdout.

Defensive by design — unknown shapes degrade to a note, never crash the step.
"""
import json, sys

MAX_BLOCK = 1500  # truncate long tool inputs/outputs to keep summary small


def trunc(s, n=MAX_BLOCK):
    s = s if isinstance(s, str) else json.dumps(s, indent=2, default=str)
    return s if len(s) <= n else s[:n] + f"\n… (+{len(s)-n} chars truncated)"


def text_blocks(content):
    """content may be a string or a list of blocks; yield (kind, payload)."""
    if isinstance(content, str):
        yield ("text", content); return
    if not isinstance(content, list):
        yield ("text", str(content)); return
    for b in content:
        if not isinstance(b, dict):
            yield ("text", str(b)); continue
        t = b.get("type")
        if t == "text":
            yield ("text", b.get("text", ""))
        elif t == "tool_use":
            yield ("tool_use", b)
        elif t == "tool_result":
            yield ("tool_result", b)
        elif t == "thinking":
            yield ("thinking", b.get("thinking", ""))
        else:
            yield ("text", f"_({t} block)_")


def render(events):
    out = ["## 🤖 Claude transcript", ""]
    for ev in events:
        if not isinstance(ev, dict):
            continue
        t = ev.get("type")
        msg = ev.get("message") or {}
        if t == "assistant":
            for kind, payload in text_blocks(msg.get("content")):
                if kind == "text" and payload.strip():
                    out += [payload.strip(), ""]
                elif kind == "thinking" and payload.strip():
                    out += ["<details><summary>💭 thinking</summary>", "",
                            "```", trunc(payload), "```", "</details>", ""]
                elif kind == "tool_use":
                    name = payload.get("name", "tool")
                    out += [f"<details><summary>🔧 <code>{name}</code></summary>", "",
                            "```json", trunc(payload.get("input", {})), "```",
                            "</details>", ""]
        elif t == "user":
            # A user event with a tool_result is tool OUTPUT, not a human turn.
            blocks = list(text_blocks(msg.get("content")))
            is_tool = any(k == "tool_result" for k, _ in blocks)
            if is_tool:
                for kind, payload in blocks:
                    if kind == "tool_result":
                        c = payload.get("content", "")
                        if isinstance(c, list):
                            c = "\n".join(b.get("text", "") for b in c if isinstance(b, dict))
                        out += ["<details><summary>↳ tool result</summary>", "",
                                "```", trunc(c), "```", "</details>", ""]
            else:
                for kind, payload in blocks:
                    if kind == "text" and payload.strip():
                        out += [f"**🧑 Prompt:** {payload.strip()}", ""]
        elif t == "result":
            turns = ev.get("num_turns", "?")
            cost = ev.get("total_cost_usd")
            dur = ev.get("duration_ms")
            stop = ev.get("stop_reason") or ev.get("terminal_reason") or ev.get("subtype")
            bits = [f"{turns} turns"]
            if dur: bits.append(f"{round(dur/1000)}s")
            if cost is not None: bits.append(f"${cost:.4f}")
            if stop: bits.append(f"stop: {stop}")
            out += ["---", f"**Run summary:** " + " · ".join(bits), ""]
        elif t == "rate_limit_event":
            out += ["> ⚠️ rate-limit event during this run", ""]
    return "\n".join(out)


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "-"
    try:
        raw = sys.stdin.read() if path == "-" else open(path).read()
        events = json.loads(raw)
        if not isinstance(events, list):
            events = [events]
        print(render(events))
    except Exception as e:
        print(f"## 🤖 Claude transcript\n\n_Could not render transcript: {e}_")


if __name__ == "__main__":
    main()
