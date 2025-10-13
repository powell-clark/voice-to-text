#include "audio.h"
#include "../common/logging.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <sys/time.h>
#include <math.h>

#define SAMPLE_RATE 16000
#define CHANNELS 1
#define FRAMES_PER_BUFFER 1024
#define MAX_RECORDING_SECONDS 300  // Maximum recording duration (5 minutes)

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
            // Mark that we've hit the buffer limit
            if (space_left > 0 && !audio->buffer_full) {
                audio->buffer_full = true;
                vtt_log("Recording buffer full - max length reached (%d seconds)", MAX_RECORDING_SECONDS);

                // Notify via callback if registered
                if (audio->buffer_full_callback) {
                    audio->buffer_full_callback(audio->callback_user_data);
                }
            }
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
    audio->selected_device_index = -1;  // Default device
    audio->buffer_full = false;
    audio->buffer_full_callback = NULL;
    audio->callback_user_data = NULL;

    vtt_log("Audio system initialized (PortAudio, %d Hz)", SAMPLE_RATE);
    return 0;
}

int vtt_audio_start_recording(vtt_audio_t *audio) {
    audio->buffer_pos = 0;
    audio->recording = true;
    audio->buffer_full = false;

    PaError err;
    if (audio->selected_device_index == -1) {
        // Use default device
        err = Pa_OpenDefaultStream(&audio->stream,
                                   CHANNELS, 0,
                                   paInt16, SAMPLE_RATE,
                                   FRAMES_PER_BUFFER,
                                   audio_callback, audio);
    } else {
        // Use selected device
        PaStreamParameters inputParams;
        inputParams.device = audio->selected_device_index;
        inputParams.channelCount = CHANNELS;
        inputParams.sampleFormat = paInt16;
        inputParams.suggestedLatency = Pa_GetDeviceInfo(audio->selected_device_index)->defaultLowInputLatency;
        inputParams.hostApiSpecificStreamInfo = NULL;

        err = Pa_OpenStream(&audio->stream,
                           &inputParams,
                           NULL,  // no output
                           SAMPLE_RATE,
                           FRAMES_PER_BUFFER,
                           paClipOff,
                           audio_callback,
                           audio);
    }

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
    bool was_buffer_full = audio->buffer_full;

    if (audio->stream) {
        Pa_StopStream(audio->stream);
        Pa_CloseStream(audio->stream);
        audio->stream = NULL;
    }

    float duration = (float)audio->buffer_pos / SAMPLE_RATE;

    // Check minimum duration
    if (duration < 0.5) {
        vtt_log("Recording too short (%.2fs)", duration);
        // Return special marker for "too short"
        char *marker = malloc(256);
        snprintf(marker, 256, "REJECTED:TOO_SHORT:%.2f", duration);
        return marker;
    }

    // Check amplitude
    int max_amp = 0;
    for (int i = 0; i < audio->buffer_pos; i++) {
        int amp = abs(audio->buffer[i]);
        if (amp > max_amp) max_amp = amp;
    }

    if (max_amp < 500) {
        vtt_log("Audio too quiet (amplitude %d)", max_amp);
        // Return special marker for "too quiet"
        char *marker = malloc(256);
        snprintf(marker, 256, "REJECTED:TOO_QUIET:%d", max_amp);
        return marker;
    }

    vtt_log("Recording stopped: %.2fs%s, amplitude: %d", duration,
            was_buffer_full ? " (MAX LENGTH REACHED)" : "", max_amp);

    // Save to WAV file with microsecond precision to prevent collisions
    char *filename = malloc(256);
    struct timeval tv;
    gettimeofday(&tv, NULL);
    snprintf(filename, 256, "/tmp/vtt_recording_%ld_%06ld.wav",
             (long)tv.tv_sec, (long)tv.tv_usec);

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

    // If buffer was full, prepend marker to indicate max length reached
    if (was_buffer_full) {
        char *marked_filename = malloc(512);
        snprintf(marked_filename, 512, "MAX_LENGTH_REACHED:%s", filename);
        free(filename);
        return marked_filename;
    }

    return filename;
}

vtt_audio_device_t **vtt_audio_get_devices(int *count) {
    int numDevices = Pa_GetDeviceCount();
    if (numDevices < 0) {
        *count = 0;
        return NULL;
    }

    // Count input devices
    int inputCount = 0;
    for (int i = 0; i < numDevices; i++) {
        const PaDeviceInfo *info = Pa_GetDeviceInfo(i);
        if (info && info->maxInputChannels > 0) {
            inputCount++;
        }
    }

    if (inputCount == 0) {
        *count = 0;
        return NULL;
    }

    // Allocate device array
    vtt_audio_device_t **devices = malloc(inputCount * sizeof(vtt_audio_device_t *));
    int defaultDevice = Pa_GetDefaultInputDevice();
    int idx = 0;

    for (int i = 0; i < numDevices; i++) {
        const PaDeviceInfo *info = Pa_GetDeviceInfo(i);
        if (info && info->maxInputChannels > 0) {
            devices[idx] = malloc(sizeof(vtt_audio_device_t));
            devices[idx]->name = strdup(info->name);
            devices[idx]->index = i;
            devices[idx]->is_default = (i == defaultDevice);
            idx++;
        }
    }

    *count = inputCount;
    return devices;
}

void vtt_audio_free_devices(vtt_audio_device_t **devices, int count) {
    if (!devices) return;
    for (int i = 0; i < count; i++) {
        if (devices[i]) {
            free(devices[i]->name);
            free(devices[i]);
        }
    }
    free(devices);
}

void vtt_audio_set_device(vtt_audio_t *audio, int device_index) {
    audio->selected_device_index = device_index;
    if (device_index == -1) {
        vtt_log("Audio device set to default");
    } else {
        const PaDeviceInfo *info = Pa_GetDeviceInfo(device_index);
        if (info) {
            vtt_log("Audio device set to: %s (index %d)", info->name, device_index);
        }
    }
}

void vtt_audio_set_buffer_full_callback(vtt_audio_t *audio,
                                         vtt_audio_buffer_full_callback_t callback,
                                         void *user_data) {
    audio->buffer_full_callback = callback;
    audio->callback_user_data = user_data;
}

void vtt_audio_cleanup(vtt_audio_t *audio) {
    if (audio->stream) {
        Pa_CloseStream(audio->stream);
    }
    Pa_Terminate();
    free(audio->buffer);
}
