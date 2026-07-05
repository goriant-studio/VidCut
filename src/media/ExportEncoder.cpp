#include "ExportEncoder.h"

#include <QProcess>
#include <QTemporaryFile>
#include <QTextStream>
#include <QFile>
#include <QDir>
#include <QFileInfo>
#include <QStandardPaths>
#include <QRegularExpression>
#include <QDateTime>
#include <QCoreApplication>
#include <QDebug>

#include "vidcut/Timeline.h"
#include "vidcut/Track.h"
#include "vidcut/Clip.h"
#include "vidcut/MediaAsset.h"

namespace VidCut {

ExportEncoder::ExportEncoder(QObject* parent) : QObject(parent) {}

ExportEncoder::~ExportEncoder() {
    cancel();
}

// ── Find ffmpeg.exe ──────────────────────────────────────────────────────────
QString ExportEncoder::findFfmpegExe() const {
    // Common locations
    const QStringList candidates = {
        "ffmpeg",           // PATH
        "ffmpeg.exe",
        "C:/msys64/mingw64/bin/ffmpeg.exe",
        "C:/ffmpeg/bin/ffmpeg.exe",
        "C:/Program Files/ffmpeg/bin/ffmpeg.exe",
        QDir::homePath() + "/ffmpeg/bin/ffmpeg.exe",
        QCoreApplication::applicationDirPath() + "/ffmpeg.exe",
    };
    for (const QString& c : candidates) {
        QFileInfo fi(c);
        if (fi.exists() && fi.isExecutable()) return c;
        // Try via PATH lookup
        if (!c.contains('/') && !c.contains('\\')) {
            QProcess probe;
            probe.start(c, {"--version"});
            if (probe.waitForStarted(500)) { probe.kill(); return c; }
        }
    }
    return {};
}

// ── Build ffmpeg concat script ───────────────────────────────────────────────
QString ExportEncoder::buildConcatScript(VidCut::Timeline* timeline) const {
    if (!timeline) return {};

    // Collect all video clips sorted by timelineStartMs
    // For simplicity: build a concat list of the source files
    // (in the future: apply trim with -ss/-to per segment)
    QString script;
    QTextStream ts(&script);

    // Gather all clips across all tracks, ordered by timeline position
    struct ClipEntry { qint64 startMs; qint64 srcStartMs; qint64 srcEndMs; QString path; };
    QList<ClipEntry> entries;

    for (int ti = 0; ti < timeline->trackCount(); ++ti) {
        auto* track = timeline->track(ti);
        if (track->type() != TrackType::Video) continue;
        for (const auto& clip : track->clips()) {
            if (!clip.asset) continue;
            entries << ClipEntry{
                clip.timelineStartMs,
                clip.srcStartMs,
                clip.srcEndMs,
                clip.asset->filePath
            };
        }
    }

    if (entries.isEmpty()) return {};

    // Sort by timeline position
    std::sort(entries.begin(), entries.end(),
        [](const ClipEntry& a, const ClipEntry& b){ return a.startMs < b.startMs; });

    ts << "ffconcat version 1.0\n";
    for (const auto& e : entries) {
        double durSec = (e.srcEndMs - e.srcStartMs) / 1000.0;
        ts << "file " << "'" << e.path << "'\n";
        if (e.srcStartMs > 0)
            ts << "inpoint " << (e.srcStartMs / 1000.0) << "\n";
        ts << "outpoint " << (e.srcEndMs / 1000.0) << "\n";
        ts << "duration " << durSec << "\n";
    }

    return script;
}

// ── startExport ──────────────────────────────────────────────────────────────
void ExportEncoder::startExport(const ExportSettings& settings, VidCut::Timeline* timeline) {
    m_settings  = settings;
    m_cancelled = false;
    m_totalDurationMs = timeline ? timeline->durationMs() : 0;

    QString ffmpeg = findFfmpegExe();
    if (ffmpeg.isEmpty()) {
        emit exportFinished(false, settings.outputPath);
        qWarning() << "ffmpeg not found. Install ffmpeg and add to PATH.";
        return;
    }

    // Write concat script to temp file
    QString concatScript = buildConcatScript(timeline);
    if (concatScript.isEmpty()) {
        emit exportFinished(false, settings.outputPath);
        return;
    }

    // Save concat list to a temp file
    m_concatFile = QDir::tempPath() + "/vidcut_concat_" +
                   QString::number(QDateTime::currentMSecsSinceEpoch()) + ".txt";
    QFile f(m_concatFile);
    if (!f.open(QIODevice::WriteOnly | QIODevice::Text)) {
        emit exportFinished(false, settings.outputPath);
        return;
    }
    f.write(concatScript.toUtf8());
    f.close();

    // Build ffmpeg args
    // -f concat -safe 0 -i <concatfile> -c:v libx264 -b:v <br> -c:a aac -b:a <br> -y <output>
    QStringList args;
    args << "-f" << "concat"
         << "-safe" << "0"
         << "-i" << m_concatFile
         << "-c:v" << "libx264"
         << "-preset" << "fast"
         << "-b:v" << QString::number(settings.videoBitrate)
         << "-vf" << QString("scale=%1:%2").arg(settings.width).arg(settings.height)
         << "-r" << QString::number(settings.fps)
         << "-c:a" << "aac"
         << "-b:a" << QString::number(settings.audioBitrate)
         << "-movflags" << "+faststart"
         << "-y"
         << settings.outputPath;

    m_process = new QProcess(this);
    m_process->setProcessChannelMode(QProcess::MergedChannels);

    connect(m_process, &QProcess::readyReadStandardOutput,
            this, &ExportEncoder::onProcessOutput);
    connect(m_process, QOverload<int,QProcess::ExitStatus>::of(&QProcess::finished),
            this, [this](int code, QProcess::ExitStatus) { onProcessFinished(code); });

    emit progressChanged(0);
    m_process->start(ffmpeg, args);
}

void ExportEncoder::cancel() {
    m_cancelled = true;
    if (m_process && m_process->state() != QProcess::NotRunning) {
        m_process->kill();
    }
}

// ── Parse ffmpeg progress output ─────────────────────────────────────────────
void ExportEncoder::onProcessOutput() {
    if (!m_process) return;
    QByteArray out = m_process->readAll();
    QString text = QString::fromUtf8(out);

    // ffmpeg prints: time=HH:MM:SS.cc
    static QRegularExpression re("time=(\\d+):(\\d+):(\\d+\\.?\\d*)");
    auto match = re.match(text);
    if (match.hasMatch()) {
        double hh = match.captured(1).toDouble();
        double mm = match.captured(2).toDouble();
        double ss = match.captured(3).toDouble();
        qint64 currentMs = qint64((hh * 3600 + mm * 60 + ss) * 1000);
        if (m_totalDurationMs > 0) {
            int pct = int(100.0 * currentMs / m_totalDurationMs);
            emit progressChanged(qMin(99, pct));
        }
    }
}

void ExportEncoder::onProcessFinished(int exitCode) {
    // Clean up concat file
    QFile::remove(m_concatFile);

    bool success = (exitCode == 0) && !m_cancelled;
    emit progressChanged(success ? 100 : 0);
    emit exportFinished(success, m_settings.outputPath);

    if (m_process) {
        m_process->deleteLater();
        m_process = nullptr;
    }
}

} // namespace VidCut
