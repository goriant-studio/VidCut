#pragma once
#include <QObject>
#include <QImage>
#include <QString>

namespace VidCut {

// Asynchronously extracts a frame at a given timestamp for use as a clip thumbnail.
// Phase 1: stub only.
class ThumbnailGenerator : public QObject {
    Q_OBJECT
public:
    explicit ThumbnailGenerator(QObject* parent = nullptr);

    // Request a thumbnail for filePath at positionMs.
    // Emits thumbnailReady when done.
    void requestThumbnail(const QString& filePath, qint64 positionMs);

signals:
    void thumbnailReady(const QString& filePath, qint64 positionMs, const QImage& image);

private:
    // TODO: run in a background QThread
};

} // namespace VidCut
