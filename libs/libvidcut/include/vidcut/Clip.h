#pragma once
#include <QString>

namespace VidCut {

class MediaAsset;

// A Clip is a reference to a portion of a MediaAsset placed on the timeline.
struct Clip {
    QString id;
    MediaAsset* asset = nullptr;

    // Source range (within the media file)
    qint64 srcStartMs = 0;
    qint64 srcEndMs = 0;

    // Position on the timeline track
    qint64 timelineStartMs = 0;

    qint64 durationMs() const { return srcEndMs - srcStartMs; }
};

} // namespace VidCut
