#include "AudioDecoder.h"

namespace VidCut {

AudioDecoder::AudioDecoder(QObject* parent) : QObject(parent) {}

bool AudioDecoder::open(const QString& filePath) {
    Q_UNUSED(filePath)
    // TODO: open audio stream via libavformat + libswresample
    m_filePath = filePath;
    return false;
}

void AudioDecoder::close() {
    m_isOpen = false;
    m_filePath.clear();
}

QVector<float> AudioDecoder::generateWaveform(int numSamples) {
    Q_UNUSED(numSamples)
    // TODO: decode audio, compute RMS per block, return peaks
    return {};
}

} // namespace VidCut
