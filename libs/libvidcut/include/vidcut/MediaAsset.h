#pragma once
#include <QString>

namespace VidCut {

// Represents an imported media file in the media pool.
struct MediaAsset {
    QString id;
    QString filePath;
    QString name;

    int width = 0;
    int height = 0;
    qint64 durationMs = 0;
    double fps = 0.0;
    bool hasVideo = false;
    bool hasAudio = false;
};

} // namespace VidCut
