#pragma once
#include <QString>
#include <QObject>

// Forward declare FFmpeg types to avoid pulling headers into the interface
struct AVFormatContext;
struct AVCodecContext;

namespace VidCut {

struct MediaAsset;

// Wraps FFmpeg to open a media file and decode frames.
// Phase 1: stub only.
class MediaDecoder : public QObject {
    Q_OBJECT
public:
    explicit MediaDecoder(QObject* parent = nullptr);
    ~MediaDecoder() override;

    bool open(const QString& filePath);
    void close();

    bool isOpen() const { return m_formatCtx != nullptr; }
    QString filePath() const { return m_filePath; }

signals:
    void errorOccurred(const QString& message);

private:
    QString m_filePath;
    AVFormatContext* m_formatCtx = nullptr;
    AVCodecContext* m_videoCtx = nullptr;
};

} // namespace VidCut
