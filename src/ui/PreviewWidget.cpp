#include "PreviewWidget.h"

#include <QPainter>
#include <QPainterPath>
#include <QUrl>
#include <QSizePolicy>
#include <QFrame>
#include <QStyle>

// ── Constructor ──────────────────────────────────────────────────────────────
PreviewWidget::PreviewWidget(QWidget* parent) : QWidget(parent) {
    setObjectName("previewWidget");
    setMinimumSize(480, 300);
    setupUI();
}

void PreviewWidget::setupUI() {
    // Media player
    m_player = new QMediaPlayer(this);
    m_audio  = new QAudioOutput(this);
    m_audio->setVolume(0.8f);
    m_player->setAudioOutput(m_audio);

    // Video widget (renders actual video frames)
    m_video = new QVideoWidget(this);
    m_video->setObjectName("videoWidget");
    m_video->setMinimumSize(320, 180);
    m_video->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
    m_player->setVideoOutput(m_video);

    // "No media" overlay label
    m_noMedia = new QLabel("No media loaded\nImport files and drag to timeline", m_video);
    m_noMedia->setObjectName("noMediaLabel");
    m_noMedia->setAlignment(Qt::AlignCenter);
    m_noMedia->setWordWrap(true);
    m_noMedia->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);

    // ── Transport Controls ───────────────────────────────────────────────────

    // Seek slider
    m_seekSlider = new QSlider(Qt::Horizontal, this);
    m_seekSlider->setObjectName("seekSlider");
    m_seekSlider->setRange(0, 0);
    m_seekSlider->setSingleStep(1000);
    m_seekSlider->setPageStep(5000);

    // Volume slider
    m_volumeSlider = new QSlider(Qt::Horizontal, this);
    m_volumeSlider->setObjectName("volumeSlider");
    m_volumeSlider->setRange(0, 100);
    m_volumeSlider->setValue(80);
    m_volumeSlider->setMaximumWidth(80);
    m_volumeSlider->setToolTip("Volume");

    // Buttons
    m_btnStart = new QPushButton("⏮", this);
    m_btnPlay  = new QPushButton("▶", this);
    m_btnStop  = new QPushButton("⏹", this);
    m_btnEnd   = new QPushButton("⏭", this);

    for (auto* btn : {m_btnStart, m_btnPlay, m_btnStop, m_btnEnd}) {
        btn->setObjectName("transportBtn");
        btn->setFixedSize(36, 30);
        btn->setCursor(Qt::PointingHandCursor);
    }
    m_btnPlay->setObjectName("playBtn");
    m_btnPlay->setFixedSize(44, 32);

    // Timecode labels
    m_timecode = new QLabel("00:00:00", this);
    m_timecode->setObjectName("timecodeLabel");
    m_duration = new QLabel("/ 00:00:00", this);
    m_duration->setObjectName("durationLabel");

    // ── Layout ───────────────────────────────────────────────────────────────

    // Seek bar row
    auto* seekRow = new QHBoxLayout();
    seekRow->setContentsMargins(8, 0, 8, 0);
    seekRow->addWidget(m_seekSlider);

    // Transport row
    auto* transportRow = new QHBoxLayout();
    transportRow->setContentsMargins(8, 2, 8, 2);
    transportRow->setSpacing(4);
    transportRow->addWidget(m_btnStart);
    transportRow->addWidget(m_btnPlay);
    transportRow->addWidget(m_btnStop);
    transportRow->addWidget(m_btnEnd);
    transportRow->addSpacing(8);
    transportRow->addWidget(m_timecode);
    transportRow->addWidget(m_duration);
    transportRow->addStretch();
    transportRow->addWidget(new QLabel("🔊", this));
    transportRow->addWidget(m_volumeSlider);

    // Controls panel
    auto* controlsFrame = new QFrame(this);
    controlsFrame->setObjectName("controlsFrame");
    auto* controlsLayout = new QVBoxLayout(controlsFrame);
    controlsLayout->setContentsMargins(0, 4, 0, 4);
    controlsLayout->setSpacing(2);
    controlsLayout->addLayout(seekRow);
    controlsLayout->addLayout(transportRow);

    // Main layout
    auto* mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);
    mainLayout->addWidget(m_video, 1);
    mainLayout->addWidget(controlsFrame, 0);

    // ── Signal Connections ────────────────────────────────────────────────────
    connect(m_player, &QMediaPlayer::positionChanged,
            this,     &PreviewWidget::onPlayerPositionChanged);
    connect(m_player, &QMediaPlayer::durationChanged,
            this,     &PreviewWidget::onDurationChanged);
    connect(m_player, &QMediaPlayer::playbackStateChanged,
            this,     &PreviewWidget::onPlayerStateChanged);

    connect(m_seekSlider, &QSlider::sliderPressed,  this, [this]{ m_sliderDragging = true; });
    connect(m_seekSlider, &QSlider::sliderReleased, this, [this]{
        m_sliderDragging = false;
        m_player->setPosition(m_seekSlider->value());
    });
    connect(m_seekSlider, &QSlider::sliderMoved, this, &PreviewWidget::onSliderMoved);

    connect(m_volumeSlider, &QSlider::valueChanged, this, &PreviewWidget::onVolumeChanged);

    connect(m_btnStart, &QPushButton::clicked, this, &PreviewWidget::goToStart);
    connect(m_btnPlay,  &QPushButton::clicked, this, &PreviewWidget::togglePlay);
    connect(m_btnStop,  &QPushButton::clicked, this, &PreviewWidget::stop);
    connect(m_btnEnd,   &QPushButton::clicked, this, &PreviewWidget::goToEnd);
}

// ── Public API ────────────────────────────────────────────────────────────────
void PreviewWidget::loadAsset(VidCut::MediaAsset* asset) {
    if (!asset) return;
    m_player->stop();
    m_player->setSource(QUrl::fromLocalFile(asset->filePath));
    m_noMedia->setVisible(false);
    m_player->pause(); // prepare without playing
}

void PreviewWidget::seekTo(qint64 ms) {
    if (m_player->source().isEmpty()) return;
    m_player->setPosition(ms);
}

bool PreviewWidget::isPlaying() const {
    return m_player->playbackState() == QMediaPlayer::PlayingState;
}

void PreviewWidget::play()       { m_player->play(); }
void PreviewWidget::pause()      { m_player->pause(); }
void PreviewWidget::stop()       { m_player->stop(); emit positionChanged(0); }
void PreviewWidget::goToStart()  { m_player->setPosition(0); }
void PreviewWidget::goToEnd()    { if (m_durationMs > 0) m_player->setPosition(m_durationMs); }

void PreviewWidget::togglePlay() {
    if (m_player->playbackState() == QMediaPlayer::PlayingState)
        m_player->pause();
    else
        m_player->play();
}

// ── Player callbacks ──────────────────────────────────────────────────────────
void PreviewWidget::onPlayerPositionChanged(qint64 pos) {
    updateTimecodeLabel(pos);
    if (!m_sliderDragging)
        m_seekSlider->setValue(int(pos));
    emit positionChanged(pos);
}

void PreviewWidget::onDurationChanged(qint64 dur) {
    m_durationMs = dur;
    m_seekSlider->setRange(0, int(dur));
    m_duration->setText("/ " + msToTimecode(dur));
}

void PreviewWidget::onPlayerStateChanged(QMediaPlayer::PlaybackState state) {
    if (state == QMediaPlayer::PlayingState)
        m_btnPlay->setText("⏸");
    else
        m_btnPlay->setText("▶");
}

void PreviewWidget::onSliderMoved(int value) {
    updateTimecodeLabel(value);
}

void PreviewWidget::onVolumeChanged(int value) {
    m_audio->setVolume(value / 100.0f);
}

// ── Helpers ───────────────────────────────────────────────────────────────────
void PreviewWidget::updateTimecodeLabel(qint64 ms) {
    m_timecode->setText(msToTimecode(ms));
}

QString PreviewWidget::msToTimecode(qint64 ms) const {
    qint64 total = ms / 1000;
    int hh = total / 3600;
    int mm = (total % 3600) / 60;
    int ss = total % 60;
    if (hh > 0)
        return QString("%1:%2:%3")
            .arg(hh).arg(mm, 2, 10, QChar('0')).arg(ss, 2, 10, QChar('0'));
    return QString("%1:%2")
        .arg(mm, 2, 10, QChar('0')).arg(ss, 2, 10, QChar('0'));
}
