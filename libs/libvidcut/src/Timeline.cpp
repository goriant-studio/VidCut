#include "vidcut/Timeline.h"
#include "vidcut/Track.h"

namespace VidCut {

void Timeline::addTrack(Track* track) {
    m_tracks.append(track);
}

void Timeline::removeTrack(int index) {
    if (index >= 0 && index < m_tracks.size())
        m_tracks.removeAt(index);
}

Track* Timeline::track(int index) const {
    if (index >= 0 && index < m_tracks.size())
        return m_tracks[index];
    return nullptr;
}

qint64 Timeline::durationMs() const {
    qint64 maxEnd = 0;
    for (auto* t : m_tracks) {
        for (const auto& clip : t->clips())
            maxEnd = qMax(maxEnd, clip.timelineStartMs + clip.durationMs());
    }
    return maxEnd;
}

} // namespace VidCut
