#pragma once
#include <QObject>
#include "vidcut/MediaAsset.h"

namespace VidCut {

// QObject-based signal emitter for MediaPool.
// Lives in src/ (executable target) so Qt MOC works correctly.
class MediaPoolSignals : public QObject {
    Q_OBJECT
public:
    explicit MediaPoolSignals(QObject* parent = nullptr) : QObject(parent) {}
signals:
    void assetAdded(VidCut::MediaAsset* asset);
    void assetRemoved(const QString& id);
};

} // namespace VidCut
