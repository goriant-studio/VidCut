#include "vidcut/Track.h"

namespace VidCut {

Track::Track(TrackType type, const QString& name)
    : m_type(type), m_name(name) {}

void Track::addClip(const Clip& clip) {
    m_clips.append(clip);
}

void Track::removeClip(const QString& clipId) {
    m_clips.removeIf([&](const Clip& c) { return c.id == clipId; });
}

} // namespace VidCut
