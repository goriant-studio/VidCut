#pragma once
#include <QWidget>
#include <QMediaPlayer>
#include <QAudioOutput>
#include <QVideoWidget>
#include <QSlider>
#include <QLabel>
#include <QPushButton>
#include <QHBoxLayout>
#include <QVBoxLayout>

#include "vidcut/MediaAsset.h"

class PreviewWidget : public QWidget {
    Q_OBJECT
public:
    explicit PreviewWidget(QWidget* parent = nullptr);
    ~PreviewWidget() override = default;

    // Load an asset into the player
    void loadAsset(VidCut::MediaAsset* asset);

    // Seek to position (called by timeline playhead)
    void seekTo(qint64 ms);

    bool isPlaying() const;

public slots:
    void play();
    void pause();
    void togglePlay();
    void stop();
    void goToStart();
    void goToEnd();

signals:
    // Emitted as the player advances — connect to TimelineWidget::setPlayheadMs
    void positionChanged(qint64 ms);

private slots:
    void onPlayerPositionChanged(qint64 pos);
    void onDurationChanged(qint64 dur);
    void onPlayerStateChanged(QMediaPlayer::PlaybackState state);
    void onSliderMoved(int value);
    void onVolumeChanged(int value);

private:
    void setupUI();
    void updateTimecodeLabel(qint64 ms);
    QString msToTimecode(qint64 ms) const;

    // Media
    QMediaPlayer*  m_player  = nullptr;
    QAudioOutput*  m_audio   = nullptr;
    QVideoWidget*  m_video   = nullptr;

    // Controls
    QSlider*       m_seekSlider   = nullptr;
    QSlider*       m_volumeSlider = nullptr;
    QPushButton*   m_btnStart = nullptr;
    QPushButton*   m_btnPlay  = nullptr;
    QPushButton*   m_btnStop  = nullptr;
    QPushButton*   m_btnEnd   = nullptr;
    QLabel*        m_timecode = nullptr;
    QLabel*        m_duration = nullptr;
    QLabel*        m_noMedia  = nullptr;

    bool m_sliderDragging = false;
    qint64 m_durationMs = 0;
};
