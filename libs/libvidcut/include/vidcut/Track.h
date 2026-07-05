#pragma once
#include <QString>
#include <QList>
#include "Clip.h"

namespace VidCut {

enum class TrackType { Video, Audio };

class Track {
public:
    explicit Track(TrackType type, const QString& name);

    TrackType type() const { return m_type; }
    QString name() const { return m_name; }

    void addClip(const Clip& clip);
    void removeClip(const QString& clipId);

    const QList<Clip>& clips() const { return m_clips; }
    QList<Clip>& clips() { return m_clips; }

private:
    TrackType m_type;
    QString m_name;
    QList<Clip> m_clips;
};

} // namespace VidCut
