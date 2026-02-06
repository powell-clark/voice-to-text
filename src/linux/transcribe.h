#ifndef VTT_TRANSCRIBE_LINUX_H
#define VTT_TRANSCRIBE_LINUX_H

// Transcribe audio file and return text (caller must free)
// language: "en" for English (fastest), "auto" for auto-detect (99 languages)
// initial_prompt: vocabulary hint for Whisper decoder (NULL for default)
char *vtt_transcribe_audio(const char *audio_path, const char *model, const char *language, const char *initial_prompt);

#endif // VTT_TRANSCRIBE_LINUX_H
