#ifndef VTT_TRANSCRIBE_LINUX_H
#define VTT_TRANSCRIBE_LINUX_H

// Transcribe audio file and return text (caller must free)
char *vtt_transcribe_audio(const char *audio_path);

#endif // VTT_TRANSCRIBE_LINUX_H
