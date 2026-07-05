#include "vidcut/MediaPool.h"
#include "vidcut/MediaAsset.h"

#include <QMediaPlayer>
#include <QMediaMetaData>
#include <QAudioOutput>
#include <QUrl>
#include <QFileInfo>
#include <QUuid>
#include <QEventLoop>
#include <QTimer>
#include <QSize>

namespace VidCut {

MediaPool::MediaPool()
    : m_notifier(nullptr)
{}

MediaPool::~MediaPool() {
    qDeleteAll(m_assets);
}

MediaAsset* MediaPool::importFile(const QString& filePath) {
    QFileInfo fi(filePath);
    if (!fi.exists() || !fi.isFile()) return nullptr;

    QMediaPlayer player;
    QAudioOutput audioOut;
    player.setAudioOutput(&audioOut);
    player.setSource(QUrl::fromLocalFile(filePath));

    // Wait for LoadedMedia (metadata available) or error, max 5 s
    QEventLoop loop;
    bool ready = false;

    QObject::connect(&player, &QMediaPlayer::mediaStatusChanged,
        [&](QMediaPlayer::MediaStatus status) {
            if (status == QMediaPlayer::LoadedMedia  ||
                status == QMediaPlayer::BufferedMedia ||
                status == QMediaPlayer::EndOfMedia) {
                if (!ready) { ready = true; loop.quit(); }
            }
        });
    QObject::connect(&player, &QMediaPlayer::errorOccurred,
        [&](QMediaPlayer::Error, const QString&) {
            if (!ready) { ready = true; loop.quit(); }
        });

    QTimer::singleShot(5000, &loop, &QEventLoop::quit);
    loop.exec();

    auto* asset = new MediaAsset();
    asset->id         = QUuid::createUuid().toString(QUuid::WithoutBraces);
    asset->filePath   = filePath;
    asset->name       = fi.fileName();
    asset->durationMs = player.duration();
    asset->hasVideo   = player.hasVideo();
    asset->hasAudio   = player.hasAudio();

    QMediaMetaData meta = player.metaData();

    QSize resolution = meta.value(QMediaMetaData::Resolution).toSize();
    if (!resolution.isEmpty()) {
        asset->width  = resolution.width();
        asset->height = resolution.height();
    } else {
        asset->width  = 1920;
        asset->height = 1080;
    }

    double fps = meta.value(QMediaMetaData::VideoFrameRate).toDouble();
    asset->fps = (fps > 0.0) ? fps : 30.0;

    if (asset->durationMs <= 0 && !asset->hasVideo && !asset->hasAudio) {
        delete asset;
        return nullptr;
    }
    if (asset->durationMs <= 0)
        asset->durationMs = 10000;

    m_assets.append(asset);
    return asset;
}

void MediaPool::removeAsset(const QString& id) {
    for (int i = 0; i < m_assets.size(); ++i) {
        if (m_assets[i]->id == id) {
            auto* a = m_assets.takeAt(i);
            delete a;
            return;
        }
    }
}

MediaAsset* MediaPool::findById(const QString& id) const {
    for (auto* a : m_assets)
        if (a->id == id) return a;
    return nullptr;
}

} // namespace VidCut
