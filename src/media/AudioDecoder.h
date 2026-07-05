#pragma once
#include <QObject>
#include <QVector>
#include <QString>

namespace VidCut {

// Decodes audio samples and generates a waveform for timeline display.
// Phase 1: stub only.
class AudioDecoder : public QObject {
    Q_OBJECT
public:
    explicit AudioDecoder(QObject* parent = nullptr);

    bool open(const QString& filePath);
    void close();

    // Returns normalised peak values [0.0, 1.0] for waveform rendering.
    QVector<float> generateWaveform(int numSamples);

private:
    QString m_filePath;
    bool m_isOpen = false;
};

} // namespace VidCut
