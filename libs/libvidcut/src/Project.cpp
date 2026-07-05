#include "vidcut/Project.h"
#include "vidcut/Timeline.h"

namespace VidCut {

Project Project::create(const QString& name, ProjectSettings settings) {
    Project p;
    p.m_name = name;
    p.m_settings = settings;
    p.m_timeline = new Timeline();
    // TODO: initialize MediaPool
    return p;
}

Project Project::load(const QString& path) {
    Q_UNUSED(path)
    // TODO: parse .vidcut XML file
    return Project();
}

bool Project::save(const QString& path) const {
    Q_UNUSED(path)
    // TODO: serialize to .vidcut XML
    return false;
}

} // namespace VidCut
