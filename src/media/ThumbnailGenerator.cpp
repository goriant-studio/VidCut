#include "ThumbnailGenerator.h"

#include <QThread>
#include <QRunnable>
#include <QThreadPool>
#include <QMediaPlayer>
#include <QAudioOutput>
#include <QVideoSink>
#include <QVideoFrame>
#include <QImage>
#include <QUrl>
#include <QEventLoop>
#include <QTimer>

namespace VidCut {

// Worker that runs on a thread pool
class ThumbnailWorker : public QObject, public QRunnable {
    Q_OBJECT
public:
    QString filePath;
    qint64  positionMs;

    void run() override {
        QMediaPlayer player;
        QAudioOutput audioOut;
        QVideoSink   sink;
        player.setAudioOutput(&audioOut);
        player.setVideoSink(&sink);
        player.setSource(QUrl::fromLocalFile(filePath));

        QEventLoop loop;
        bool done = false;
        QImage captured;

        // Wait for media to be ready, then seek
        QObject::connect(&player, &QMediaPlayer::mediaStatusChanged,
            [&](QMediaPlayer::MediaStatus status) {
                if (status == QMediaPlayer::LoadedMedia ||
                    status == QMediaPlayer::BufferedMedia) {
                    qint64 target = qMin(positionMs, player.duration() > 0 ? player.duration() / 2 : 1000LL);
                    player.setPosition(target);
                    player.play();
                }
            });

        // Capture the first rendered frame
        QObject::connect(&sink, &QVideoSink::videoFrameChanged,
            [&](const QVideoFrame& frame) {
                if (!done && frame.isValid()) {
                    QVideoFrame copy = frame;
                    copy.map(QVideoFrame::ReadOnly);
                    captured = copy.toImage().convertToFormat(QImage::Format_RGB32)
                                   .scaled(160, 90, Qt::KeepAspectRatio, Qt::SmoothTransformation);
                    copy.unmap();
                    done = true;
                    loop.quit();
                }
            });

        QObject::connect(&player, &QMediaPlayer::errorOccurred,
            [&](QMediaPlayer::Error, const QString&) {
                if (!done) { done = true; loop.quit(); }
            });

        QTimer::singleShot(6000, &loop, &QEventLoop::quit);
        loop.exec();

        player.stop();
        emit thumbnailReady(filePath, positionMs, captured);
    }

signals:
    void thumbnailReady(const QString& filePath, qint64 positionMs, const QImage& image);
};

// ── ThumbnailGenerator ────────────────────────────────────────

ThumbnailGenerator::ThumbnailGenerator(QObject* parent) : QObject(parent) {}

void ThumbnailGenerator::requestThumbnail(const QString& filePath, qint64 positionMs) {
    auto* worker = new ThumbnailWorker();
    worker->filePath   = filePath;
    worker->positionMs = positionMs;
    worker->setAutoDelete(true);

    // Forward signal from worker to this object
    connect(worker, &ThumbnailWorker::thumbnailReady,
            this,   &ThumbnailGenerator::thumbnailReady,
            Qt::QueuedConnection);

    QThreadPool::globalInstance()->start(worker);
}

} // namespace VidCut

#include "ThumbnailGenerator.moc"
