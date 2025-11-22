#!/usr/bin/env python3
"""
Transcription History Viewer - GUI Edition
Browse, search, and replay past Voice to Text recordings
"""

import os
import re
import subprocess
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
from datetime import datetime
from pathlib import Path
from typing import List, Tuple, Optional

class RecordingEntry:
    """Represents a single recording file"""
    def __init__(self, filepath: str):
        self.filepath = filepath
        self.filename = os.path.basename(filepath)
        self.timestamp = self._parse_timestamp()
        self.size = self._get_size()
        self.duration = self._get_duration()
        self.transcription = None

    def _parse_timestamp(self) -> datetime:
        """Extract timestamp from filename: vtt_recording_YYYYMMDD_HHMMSS.wav"""
        match = re.search(r'vtt_recording_(\d{8})_(\d{6})', self.filename)
        if match:
            date_str = match.group(1)
            time_str = match.group(2)
            return datetime.strptime(f"{date_str}{time_str}", "%Y%m%d%H%M%S")
        return datetime.fromtimestamp(os.path.getmtime(self.filepath))

    def _get_size(self) -> str:
        """Get file size in human-readable format"""
        size_bytes = os.path.getsize(self.filepath)
        for unit in ['B', 'KB', 'MB', 'GB']:
            if size_bytes < 1024.0:
                return f"{size_bytes:.1f} {unit}"
            size_bytes /= 1024.0
        return f"{size_bytes:.1f} TB"

    def _get_duration(self) -> str:
        """Get audio duration using soxi or ffprobe"""
        try:
            # Try soxi first
            result = subprocess.run(
                ['soxi', '-D', self.filepath],
                capture_output=True,
                text=True,
                timeout=2
            )
            if result.returncode == 0:
                duration = float(result.stdout.strip())
                return f"{duration:.1f}s"
        except (FileNotFoundError, subprocess.TimeoutExpired, ValueError):
            pass

        try:
            # Try ffprobe
            result = subprocess.run(
                ['ffprobe', '-v', 'error', '-show_entries', 'format=duration',
                 '-of', 'default=noprint_wrappers=1:nokey=1', self.filepath],
                capture_output=True,
                text=True,
                timeout=2
            )
            if result.returncode == 0:
                duration = float(result.stdout.strip())
                return f"{duration:.1f}s"
        except (FileNotFoundError, subprocess.TimeoutExpired, ValueError):
            pass

        return "N/A"

    def load_transcription(self, log_file: str):
        """Find transcription text in log file"""
        if not os.path.exists(log_file):
            self.transcription = "(log file not found)"
            return

        try:
            with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()

            # Find lines containing this filename
            pattern = rf'{re.escape(self.filename)}.*?Transcription:\s*(.+?)(?:\n|$)'
            match = re.search(pattern, content, re.DOTALL)

            if match:
                self.transcription = match.group(1).strip()
            else:
                self.transcription = "(not found in log)"
        except Exception as e:
            self.transcription = f"(error reading log: {e})"

    def play(self):
        """Play audio file"""
        players = ['aplay', 'paplay', 'ffplay']

        for player in players:
            try:
                if player == 'ffplay':
                    subprocess.Popen(
                        [player, '-nodisp', '-autoexit', self.filepath],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL
                    )
                else:
                    subprocess.Popen(
                        [player, self.filepath],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL
                    )
                return True
            except FileNotFoundError:
                continue

        return False


class HistoryViewerGUI:
    """Main GUI application"""
    def __init__(self, root):
        self.root = root
        self.root.title("Voice to Text - History Viewer")
        self.root.geometry("1000x600")

        self.recordings_dir = os.path.expanduser("~/.local/share/voice-to-text/recordings")
        self.log_file = os.path.expanduser("~/.local/share/voice-to-text/vtt.log")

        self.recordings: List[RecordingEntry] = []
        self.filtered_recordings: List[RecordingEntry] = []

        self._setup_ui()
        self._load_recordings()

    def _setup_ui(self):
        """Create UI components"""
        # Top toolbar
        toolbar = ttk.Frame(self.root)
        toolbar.pack(side=tk.TOP, fill=tk.X, padx=5, pady=5)

        ttk.Button(toolbar, text="Refresh", command=self._load_recordings).pack(side=tk.LEFT, padx=2)
        ttk.Button(toolbar, text="Play", command=self._play_selected).pack(side=tk.LEFT, padx=2)
        ttk.Button(toolbar, text="Delete", command=self._delete_selected).pack(side=tk.LEFT, padx=2)
        ttk.Button(toolbar, text="Export All", command=self._export_all).pack(side=tk.LEFT, padx=2)

        # Search box
        ttk.Label(toolbar, text="Search:").pack(side=tk.LEFT, padx=(20, 5))
        self.search_var = tk.StringVar()
        self.search_var.trace('w', self._on_search)
        search_entry = ttk.Entry(toolbar, textvariable=self.search_var, width=30)
        search_entry.pack(side=tk.LEFT, padx=2)

        # Status label
        self.status_var = tk.StringVar(value="Loading...")
        ttk.Label(toolbar, textvariable=self.status_var).pack(side=tk.RIGHT, padx=10)

        # Treeview for recordings list
        tree_frame = ttk.Frame(self.root)
        tree_frame.pack(side=tk.TOP, fill=tk.BOTH, expand=True, padx=5, pady=5)

        # Scrollbars
        tree_scroll_y = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL)
        tree_scroll_x = ttk.Scrollbar(tree_frame, orient=tk.HORIZONTAL)

        self.tree = ttk.Treeview(
            tree_frame,
            columns=("date", "time", "duration", "size", "filename"),
            show="headings",
            yscrollcommand=tree_scroll_y.set,
            xscrollcommand=tree_scroll_x.set
        )

        tree_scroll_y.config(command=self.tree.yview)
        tree_scroll_x.config(command=self.tree.xview)

        tree_scroll_y.pack(side=tk.RIGHT, fill=tk.Y)
        tree_scroll_x.pack(side=tk.BOTTOM, fill=tk.X)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        # Configure columns
        self.tree.heading("date", text="Date")
        self.tree.heading("time", text="Time")
        self.tree.heading("duration", text="Duration")
        self.tree.heading("size", text="Size")
        self.tree.heading("filename", text="Filename")

        self.tree.column("date", width=100)
        self.tree.column("time", width=80)
        self.tree.column("duration", width=80)
        self.tree.column("size", width=80)
        self.tree.column("filename", width=300)

        # Bind double-click to play
        self.tree.bind("<Double-1>", lambda e: self._play_selected())

        # Transcription viewer
        trans_frame = ttk.LabelFrame(self.root, text="Transcription")
        trans_frame.pack(side=tk.BOTTOM, fill=tk.BOTH, expand=False, padx=5, pady=5)

        trans_scroll = ttk.Scrollbar(trans_frame, orient=tk.VERTICAL)
        self.trans_text = tk.Text(
            trans_frame,
            height=5,
            wrap=tk.WORD,
            yscrollcommand=trans_scroll.set,
            state=tk.DISABLED
        )
        trans_scroll.config(command=self.trans_text.yview)

        trans_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self.trans_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        # Bind selection change to update transcription
        self.tree.bind("<<TreeviewSelect>>", self._on_selection_changed)

    def _load_recordings(self):
        """Load all recordings from directory"""
        self.recordings = []

        if not os.path.exists(self.recordings_dir):
            self.status_var.set("No recordings directory found")
            return

        # Find all WAV files
        wav_files = list(Path(self.recordings_dir).glob("*.wav"))

        for wav_file in wav_files:
            entry = RecordingEntry(str(wav_file))
            entry.load_transcription(self.log_file)
            self.recordings.append(entry)

        # Sort by timestamp (newest first)
        self.recordings.sort(key=lambda x: x.timestamp, reverse=True)

        self._update_tree()
        self.status_var.set(f"Found {len(self.recordings)} recording(s)")

    def _update_tree(self):
        """Update tree view with current recordings"""
        # Clear existing items
        for item in self.tree.get_children():
            self.tree.delete(item)

        # Get search term
        search_term = self.search_var.get().lower()

        # Filter recordings
        if search_term:
            self.filtered_recordings = [
                r for r in self.recordings
                if search_term in r.filename.lower()
                   or (r.transcription and search_term in r.transcription.lower())
            ]
        else:
            self.filtered_recordings = self.recordings

        # Add to tree
        for rec in self.filtered_recordings:
            self.tree.insert(
                "",
                tk.END,
                values=(
                    rec.timestamp.strftime("%Y-%m-%d"),
                    rec.timestamp.strftime("%H:%M:%S"),
                    rec.duration,
                    rec.size,
                    rec.filename
                ),
                tags=(rec.filepath,)
            )

        if search_term:
            self.status_var.set(f"Showing {len(self.filtered_recordings)} of {len(self.recordings)} recording(s)")
        else:
            self.status_var.set(f"Found {len(self.recordings)} recording(s)")

    def _on_search(self, *args):
        """Handle search input change"""
        self._update_tree()

    def _on_selection_changed(self, event):
        """Handle tree selection change"""
        selection = self.tree.selection()
        if not selection:
            self._set_transcription("")
            return

        # Get selected recording
        item = selection[0]
        filepath = self.tree.item(item, "tags")[0]

        # Find recording
        recording = next((r for r in self.recordings if r.filepath == filepath), None)

        if recording and recording.transcription:
            self._set_transcription(recording.transcription)
        else:
            self._set_transcription("(no transcription available)")

    def _set_transcription(self, text: str):
        """Update transcription text widget"""
        self.trans_text.config(state=tk.NORMAL)
        self.trans_text.delete(1.0, tk.END)
        self.trans_text.insert(1.0, text)
        self.trans_text.config(state=tk.DISABLED)

    def _get_selected_recording(self) -> Optional[RecordingEntry]:
        """Get currently selected recording"""
        selection = self.tree.selection()
        if not selection:
            return None

        item = selection[0]
        filepath = self.tree.item(item, "tags")[0]
        return next((r for r in self.recordings if r.filepath == filepath), None)

    def _play_selected(self):
        """Play selected recording"""
        recording = self._get_selected_recording()
        if not recording:
            messagebox.showwarning("No Selection", "Please select a recording to play")
            return

        if not recording.play():
            messagebox.showerror(
                "Error",
                "No audio player found.\nInstall: sudo apt install alsa-utils"
            )

    def _delete_selected(self):
        """Delete selected recording"""
        recording = self._get_selected_recording()
        if not recording:
            messagebox.showwarning("No Selection", "Please select a recording to delete")
            return

        if not messagebox.askyesno(
            "Confirm Delete",
            f"Delete recording:\n{recording.filename}?"
        ):
            return

        try:
            os.remove(recording.filepath)
            messagebox.showinfo("Success", "Recording deleted")
            self._load_recordings()
        except Exception as e:
            messagebox.showerror("Error", f"Failed to delete recording:\n{e}")

    def _export_all(self):
        """Export all recordings to a directory"""
        export_dir = filedialog.askdirectory(title="Select Export Directory")
        if not export_dir:
            return

        try:
            count = 0
            for recording in self.recordings:
                dest = os.path.join(export_dir, recording.filename)
                subprocess.run(['cp', recording.filepath, dest], check=True)
                count += 1

            messagebox.showinfo("Success", f"Exported {count} recording(s) to:\n{export_dir}")
        except Exception as e:
            messagebox.showerror("Error", f"Failed to export recordings:\n{e}")


def main():
    root = tk.Tk()
    app = HistoryViewerGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()
