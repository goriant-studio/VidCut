#pragma once
#include <QWidget>
#include <QScrollBar>
#include <QTimer>

#include "vidcut/Timeline.h"
#include "vidcut/MediaPool.h"
#include "vidcut/Clip.h"

class TimelineWidget : public QWidget {
    Q_OBJECT
public:
    explicit TimelineWidget(QWidget* parent = nullptr);

    void setTimeline(VidCut::Timeline* timeline);
    void setMediaPool(VidCut::MediaPool* pool);

    // Set playhead (in ms) — called by PreviewWidget
    void setPlayheadMs(qint64 ms);
    qint64 playheadMs() const { return m_playheadMs; }

    // Pixel-per-second zoom
    double pixelsPerSecond() const { return m_pxPerSec; }

public slots:
    void zoomIn();
    void zoomOut();
    void zoomReset();

signals:
    void playheadMoved(qint64 ms);
    void clipSelected(const QString& clipId);
    void requestAddClipToTrack(const QString& assetId, int trackIndex, qint64 timelineMs);

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void leaveEvent(QEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;
    void dragEnterEvent(QDragEnterEvent* event) override;
    void dragMoveEvent(QDragMoveEvent* event) override;
    void dropEvent(QDropEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;

private:
    // Paint helpers
    void paintRuler(QPainter& p, const QRect& r);
    void paintTrackHeader(QPainter& p, const QRect& r, const QString& name, bool isVideo);
    void paintTrackLane(QPainter& p, const QRect& r, bool isVideo);
    void paintClip(QPainter& p, const QRect& r, const QString& label, bool isVideo,
                   bool selected, bool dragging);
    void paintPlayhead(QPainter& p, int x, int height);
    void paintDropIndicator(QPainter& p, int x, int trackY, int trackH);

    // Coordinate helpers
    int   msToX(qint64 ms) const;
    qint64 xToMs(int x) const;
    int   trackTop(int trackIndex) const;
    int   trackIndexAtY(int y) const;

    // Track/Clip geometry
    struct TrackInfo { bool isVideo; int height; };
    QList<TrackInfo> trackInfoList() const;
    QRect clipRect(int trackIndex, const VidCut::Clip& clip) const;

    // Find clip under cursor
    struct HitResult { int trackIdx; QString clipId; };
    HitResult hitTest(QPoint pos) const;

    // State
    VidCut::Timeline*  m_timeline = nullptr;
    VidCut::MediaPool* m_pool     = nullptr;

    qint64  m_playheadMs = 0;
    double  m_pxPerSec   = 100.0;   // zoom: pixels per second
    int     m_scrollX    = 0;       // horizontal scroll offset in pixels
    int     m_hoverX     = -1;

    // Drag-move clip state
    bool    m_draggingClip = false;
    QString m_dragClipId;
    int     m_dragTrackIdx = -1;
    qint64  m_dragClipOrigMs = 0;
    int     m_dragOffsetPx = 0;    // mouse X offset within clip

    // Playhead drag
    bool    m_draggingPlayhead = false;

    // Drop-from-browser state
    bool    m_dropActive = false;
    int     m_dropX = 0;
    int     m_dropTrackIdx = -1;

    // Selected clip
    QString m_selectedClipId;

    // Constants
    static constexpr int kRulerH      = 28;
    static constexpr int kHeaderW     = 60;
    static constexpr int kTrackH      = 52;
    static constexpr int kAudioTrackH = 40;
    static constexpr int kMinPxPerSec = 20;
    static constexpr int kMaxPxPerSec = 500;
};
