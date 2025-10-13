#include "audio.h"
#include "../common/logging.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <math.h>

#define SAMPLE_RATE 16000
#define CHANNELS 1
#define FRAMES_PER_BUFFER 1024
#define MAX_RECORDING_SECONDS 60

static int audio_callback(const void *input, void *output,
                         unsigned long frameCount,
                         const PaStreamCallbackTimeInfo *timeInfo,
                         PaStreamCallbackFlags statusFlags,
                         void *userData) {
    vtt_audio_t *audio = (vtt_audio_t *)userData;
    const short *in = (const short *)input;

    if (audio->recording && in) {
        int samples_to_copy = frameCount;
        int space_left = audio->buffer_size - audio->buffer_pos;

        if (samples_to_copy > space_left) {
            samples_to_copy = space_left;
        }

        if (samples_to_copy > 0) {
            memcpy(&audio->buffer[audio->buffer_pos], in, samples_to_copy * sizeof(short));
            audio->buffer_pos += samples_to_copy;
        }
    }

    return paContinue;
}

int vtt_audio_init(vtt_audio_t *audio) {
    PaError err = Pa_Initialize();
    if (err != paNoError) {
        vtt_log("PortAudio init failed: %s", Pa_GetErrorText(err));
        return -1;
    }

    audio->sample_rate = SAMPLE_RATE;
    audio->buffer_size = SAMPLE_RATE * MAX_RECORDING_SECONDS;
    audio->buffer = malloc(audio->buffer_size * sizeof(short));
    audio->buffer_pos = 0;
    audio->recording = false;
    audio->stream = NULL;

    vtt_log("Audio system initialized (PortAudio, %d Hz)", SAMPLE_RATE);
    return 0;
}

int vtt_audio_start_recording(vtt_audio_t *audio) {
    audio->buffer_pos = 0;
    audio->recording = true;

    PaError err = Pa_OpenDefaultStream(&audio->stream,
                                       CHANNELS, 0,
                                       paInt16, SAMPLE_RATE,
                                       FRAMES_PER_BUFFER,
                                       audio_callback, audio);
    if (err != paNoError) {
        vtt_log("Failed to open audio stream: %s", Pa_GetErrorText(err));
        return -1;
    }

    err = Pa_StartStream(audio->stream);
    if (err != paNoError) {
        vtt_log("Failed to start audio stream: %s", Pa_GetErrorText(err));
        return -1;
    }

    vtt_log("Recording started");
    return 0;
}

char *vtt_audio_stop_recording(vtt_audio_t *audio) {
    audio->recording = false;

    if (audio->stream) {
        Pa_StopStream(audio->stream);
        Pa_CloseStream(audio->stream);
        audio->stream = NULL;
    }

    float duration = (float)audio->buffer_pos / SAMPLE_RATE;

    // Check minimum duration
    if (duration < 0.5) {
        vtt_log("Recording too short (%.2fs), ignoring", duration);
        return NULL;
    }

    // Check amplitude
    int max_amp = 0;
    for (int i = 0; i < audio->buffer_pos; i++) {
        int amp = abs(audio->buffer[i]);
        if (amp > max_amp) max_amp = amp;
    }

    if (max_amp < 500) {
        vtt_log("Audio too quiet (amplitude %d), ignoring", max_amp);
        return NULL;
    }

    vtt_log("Recording stopped: %.2fs, amplitude: %d", duration, max_amp);

    // Save to WAV file
    char *filename = malloc(256);
    snprintf(filename, 256, "/tmp/vtt_recording_%ld.wav", (long)time(NULL));

    FILE *f = fopen(filename, "wb");
    if (!f) {
        free(filename);
        return NULL;
    }

    // Write WAV header
    int data_size = audio->buffer_pos * sizeof(short);
    int file_size = 36 + data_size;

    fwrite("RIFF", 1, 4, f);
    fwrite(&file_size, 4, 1, f);
    fwrite("WAVE", 1, 4, f);
    fwrite("fmt ", 1, 4, f);

    int fmt_size = 16;
    short format = 1; // PCM
    short channels = CHANNELS;
    int sample_rate = SAMPLE_RATE;
    int byte_rate = SAMPLE_RATE * CHANNELS * 2;
    short block_align = CHANNELS * 2;
    short bits_per_sample = 16;

    fwrite(&fmt_size, 4, 1, f);
    fwrite(&format, 2, 1, f);
    fwrite(&channels, 2, 1, f);
    fwrite(&sample_rate, 4, 1, f);
    fwrite(&byte_rate, 4, 1, f);
    fwrite(&block_align, 2, 1, f);
    fwrite(&bits_per_sample, 2, 1, f);
    fwrite("data", 1, 4, f);
    fwrite(&data_size, 4, 1, f);
    fwrite(audio->buffer, sizeof(short), audio->buffer_pos, f);

    fclose(f);
    vtt_log("Saved recording to %s", filename);

    return filename;
}

void vtt_audio_cleanup(vtt_audio_t *audio) {
    if (audio->stream) {
        Pa_CloseStream(audio->stream);
    }
    Pa_Terminate();
    free(audio->buffer);
}
