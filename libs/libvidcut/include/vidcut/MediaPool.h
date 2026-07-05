#pragma once
#include "MediaAsset.h"
#include <QList>
#include <QString>
#include <QUuid>

namespace VidCut {

// Forward-declare the signal emitter (defined in src/MediaPoolSignals.h)
class MediaPoolSignals;

// Owns all MediaAsset objects for a project.
// Probes file metadata using QMediaPlayer (no FFmpeg required).
class MediaPool {
public:
    explicit MediaPool();
    ~MediaPool();

    // Import a file — probes metadata synchronously, returns pointer to owned asset.
    // Returns nullptr on failure.
    MediaAsset* importFile(const QString& filePath);

    void removeAsset(const QString& id);

    const QList<MediaAsset*>& assets() const { return m_assets; }
    MediaAsset* findById(const QString& id) const;

    // Access the signal emitter to connect Qt signals
    MediaPoolSignals* notifier() const { return m_notifier; }
    void setNotifier(MediaPoolSignals* n) { m_notifier = n; }

private:
    QList<MediaAsset*>  m_assets;
    MediaPoolSignals*   m_notifier = nullptr;
};

} // namespace VidCut
