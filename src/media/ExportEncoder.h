#pragma once
#include <QObject>
#include <QString>
#include <QProcess>

namespace VidCut {

struct ExportSettings {
    QString outputPath;
    int width  = 1920;
    int height = 1080;
    int fps    = 30;
    int videoBitrate = 8000000; // 8 Mbps
    int audioBitrate = 192000;  // 192 kbps
};

class Timeline;

// Exports the timeline to an output file.
// Strategy:
//   1. Build an FFmpeg concat input list from timeline clips
//   2. Call system ffmpeg.exe via QProcess to encode
//   This works without linking libavcodec, and is compatible with any FFmpeg install.
class ExportEncoder : public QObject {
    Q_OBJECT
public:
    explicit ExportEncoder(QObject* parent = nullptr);
    ~ExportEncoder() override;

    void startExport(const ExportSettings& settings, VidCut::Timeline* timeline);
    void cancel();

signals:
    void progressChanged(int percent);
    void exportFinished(bool success, const QString& outputPath);

private slots:
    void onProcessOutput();
    void onProcessFinished(int exitCode);

private:
    QString buildConcatScript(VidCut::Timeline* timeline) const;
    QString findFfmpegExe() const;

    QProcess* m_process = nullptr;
    ExportSettings  m_settings;
    QString         m_concatFile;
    qint64          m_totalDurationMs = 0;
    bool            m_cancelled = false;
};

} // namespace VidCut
