#pragma once
#include <QString>

namespace VidCut {

class Timeline;
class MediaPool;

struct ProjectSettings {
    int fps = 30;
    int width = 1920;
    int height = 1080;
};

class Project {
public:
    static Project create(const QString& name, ProjectSettings settings);
    static Project load(const QString& path);

    bool save(const QString& path) const;

    QString name() const { return m_name; }
    ProjectSettings settings() const { return m_settings; }

    Timeline& timeline() { return *m_timeline; }
    MediaPool& mediaPool() { return *m_mediaPool; }

private:
    Project() = default;

    QString m_name;
    ProjectSettings m_settings;
    Timeline* m_timeline = nullptr;
    MediaPool* m_mediaPool = nullptr;
};

} // namespace VidCut
