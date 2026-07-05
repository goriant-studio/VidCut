#pragma once
#include <QString>
#include <QList>

namespace VidCut {

class Track;

class Timeline {
public:
    Timeline() = default;

    void addTrack(Track* track);
    void removeTrack(int index);

    Track* track(int index) const;
    int trackCount() const { return m_tracks.size(); }

    // Timeline duration in milliseconds
    qint64 durationMs() const;

private:
    QList<Track*> m_tracks;
};

} // namespace VidCut
