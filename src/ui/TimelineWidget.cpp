#include "TimelineWidget.h"

#include <QPainter>
#include <QPainterPath>
#include <QPaintEvent>
#include <QMouseEvent>
#include <QWheelEvent>
#include <QDragEnterEvent>
#include <QDragMoveEvent>
#include <QDropEvent>
#include <QMimeData>
#include <QLinearGradient>
#include <QFont>
#include <QFontMetrics>
#include <QApplication>
#include <cmath>

#include "vidcut/Track.h"
#include "vidcut/Clip.h"
#include "vidcut/MediaAsset.h"

// MIME type (must match MediaBrowserWidget)
static const QString kAssetMime = "application/x-vidcut-asset-id";

// ── Palette ─────────────────────────────────────────────────────────────────
static const QColor kBgDeep      { 0x0e, 0x0e, 0x16 };
static const QColor kBgTrack     { 0x12, 0x12, 0x1c };
static const QColor kBgTrackAlt  { 0x0f, 0x0f, 0x18 };
static const QColor kBorder      { 0x1e, 0x1e, 0x2e };
static const QColor kRulerBg     { 0x0a, 0x0a, 0x12 };
static const QColor kTextMid     { 0x70, 0x70, 0x95 };
static const QColor kTextLo      { 0x40, 0x40, 0x60 };
static const QColor kAccent      { 0x7c, 0x6b, 0xff };
static const QColor kPlayhead    { 0xff, 0xff, 0xff };
static const QColor kClipVideoA  { 0x4a, 0x3a, 0xcc };
static const QColor kClipVideoB  { 0x6a, 0x5a, 0xff };
static const QColor kClipAudioA  { 0x1a, 0x7a, 0x70 };
static const QColor kClipAudioB  { 0x25, 0xaa, 0x9a };
static const QColor kSelected    { 0xff, 0xdd, 0x55 };
static const QColor kDropLine    { 0x55, 0xff, 0xaa };

// ── Constructor ─────────────────────────────────────────────────────────────
TimelineWidget::TimelineWidget(QWidget* parent) : QWidget(parent) {
    setObjectName("timelineWidget");
    setMinimumHeight(200);
    setMouseTracking(true);
    setAttribute(Qt::WA_OpaquePaintEvent);
    setAcceptDrops(true);
}

void TimelineWidget::setTimeline(VidCut::Timeline* timeline) {
    m_timeline = timeline;
    update();
}

void TimelineWidget::setMediaPool(VidCut::MediaPool* pool) {
    m_pool = pool;
}

void TimelineWidget::setPlayheadMs(qint64 ms) {
    if (m_playheadMs == ms) return;
    m_playheadMs = ms;
    update();
}

// ── Zoom ────────────────────────────────────────────────────────────────────
void TimelineWidget::zoomIn()    { m_pxPerSec = qMin((double)kMaxPxPerSec, m_pxPerSec * 1.25); update(); }
void TimelineWidget::zoomOut()   { m_pxPerSec = qMax((double)kMinPxPerSec, m_pxPerSec / 1.25); update(); }
void TimelineWidget::zoomReset() { m_pxPerSec = 100.0; update(); }

// ── Coord helpers ────────────────────────────────────────────────────────────
int TimelineWidget::msToX(qint64 ms) const {
    return kHeaderW + int(ms * m_pxPerSec / 1000.0) - m_scrollX;
}

qint64 TimelineWidget::xToMs(int x) const {
    return qint64((x - kHeaderW + m_scrollX) * 1000.0 / m_pxPerSec);
}

QList<TimelineWidget::TrackInfo> TimelineWidget::trackInfoList() const {
    QList<TrackInfo> list;
    if (!m_timeline) {
        // Default tracks when empty
        list << TrackInfo{true, kTrackH} << TrackInfo{true, kTrackH}
             << TrackInfo{false, kAudioTrackH} << TrackInfo{false, kAudioTrackH};
    } else {
        for (int i = 0; i < m_timeline->trackCount(); ++i) {
            auto* t = m_timeline->track(i);
            bool isVideo = (t->type() == VidCut::TrackType::Video);
            list << TrackInfo{isVideo, isVideo ? kTrackH : kAudioTrackH};
        }
    }
    return list;
}

int TimelineWidget::trackTop(int idx) const {
    auto tracks = trackInfoList();
    int y = kRulerH;
    for (int i = 0; i < idx && i < tracks.size(); ++i)
        y += tracks[i].height;
    return y;
}

int TimelineWidget::trackIndexAtY(int y) const {
    auto tracks = trackInfoList();
    int cur = kRulerH;
    for (int i = 0; i < tracks.size(); ++i) {
        if (y >= cur && y < cur + tracks[i].height) return i;
        cur += tracks[i].height;
    }
    return -1;
}

QRect TimelineWidget::clipRect(int trackIndex, const VidCut::Clip& clip) const {
    auto tracks = trackInfoList();
    if (trackIndex < 0 || trackIndex >= tracks.size()) return {};
    int y = trackTop(trackIndex);
    int h = tracks[trackIndex].height;
    int x1 = msToX(clip.timelineStartMs);
    int x2 = msToX(clip.timelineStartMs + clip.durationMs());
    bool isV = tracks[trackIndex].isVideo;
    int margin = isV ? 3 : 2;
    return QRect(x1, y + margin, x2 - x1, h - 2*margin);
}

TimelineWidget::HitResult TimelineWidget::hitTest(QPoint pos) const {
    if (!m_timeline) return {-1, {}};
    for (int ti = 0; ti < m_timeline->trackCount(); ++ti) {
        auto* track = m_timeline->track(ti);
        for (const auto& clip : track->clips()) {
            QRect r = clipRect(ti, clip);
            if (r.contains(pos)) return {ti, clip.id};
        }
    }
    return {-1, {}};
}

// ── Paint ────────────────────────────────────────────────────────────────────
void TimelineWidget::paintEvent(QPaintEvent*) {
    QPainter p(this);
    p.setRenderHint(QPainter::Antialiasing);
    p.fillRect(rect(), kBgDeep);

    const int W = width();
    const int H = height();

    // Ruler
    paintRuler(p, QRect(kHeaderW, 0, W - kHeaderW, kRulerH));

    // Tracks
    auto tracks = trackInfoList();
    int y = kRulerH;
    for (int i = 0; i < tracks.size(); ++i) {
        const auto& t = tracks[i];
        QString name;
        if (m_timeline && i < m_timeline->trackCount())
            name = m_timeline->track(i)->name();
        else {
            static const QStringList defaults{"V2","V1","A1","A2"};
            name = (i < defaults.size()) ? defaults[i] : QString("T%1").arg(i+1);
        }

        paintTrackHeader(p, QRect(0, y, kHeaderW, t.height), name, t.isVideo);
        paintTrackLane(p, QRect(kHeaderW, y, W - kHeaderW, t.height), t.isVideo);

        // Paint clips for this track
        if (m_timeline && i < m_timeline->trackCount()) {
            for (const auto& clip : m_timeline->track(i)->clips()) {
                QRect r = clipRect(i, clip);
                if (r.right() < kHeaderW || r.left() > W) continue; // off-screen
                QString label = (m_pool && clip.asset) ? clip.asset->name : clip.id;
                bool sel = (clip.id == m_selectedClipId);
                bool dragging = (clip.id == m_dragClipId && m_draggingClip);
                if (!dragging) paintClip(p, r, label, t.isVideo, sel, false);
            }
        }

        // Drop indicator
        if (m_dropActive && m_dropTrackIdx == i) {
            paintDropIndicator(p, m_dropX, y, t.height);
        }

        // Track divider
        p.setPen(QPen(kBorder, 1));
        p.drawLine(0, y + t.height, W, y + t.height);
        y += t.height;
    }

    // Dragging clip ghost
    if (m_draggingClip && m_timeline) {
        for (int ti = 0; ti < m_timeline->trackCount(); ++ti) {
            for (const auto& clip : m_timeline->track(ti)->clips()) {
                if (clip.id == m_dragClipId) {
                    QRect r = clipRect(ti, clip);
                    auto tracks2 = trackInfoList();
                    bool isV = (ti < tracks2.size()) ? tracks2[ti].isVideo : true;
                    QPainter::CompositionMode cm = p.compositionMode();
                    p.setOpacity(0.6);
                    paintClip(p, r, clip.asset ? clip.asset->name : clip.id, isV, false, true);
                    p.setOpacity(1.0);
                    p.setCompositionMode(cm);
                    break;
                }
            }
        }
    }

    // Playhead
    int phX = msToX(m_playheadMs);
    paintPlayhead(p, phX, H);

    // Hover scrub
    if (m_hoverX > kHeaderW) {
        p.setPen(QPen(QColor(255, 255, 255, 25), 1));
        p.drawLine(m_hoverX, kRulerH, m_hoverX, H);
    }

    // Header/track separator
    p.setPen(QPen(QColor(0x22, 0x22, 0x35), 1));
    p.drawLine(kHeaderW, 0, kHeaderW, H);
}

// ── Ruler ────────────────────────────────────────────────────────────────────
void TimelineWidget::paintRuler(QPainter& p, const QRect& r) {
    QLinearGradient bg(0, r.top(), 0, r.bottom());
    bg.setColorAt(0.0, QColor(0x0c, 0x0c, 0x14));
    bg.setColorAt(1.0, QColor(0x0a, 0x0a, 0x12));
    p.fillRect(r, bg);

    p.setPen(QPen(kBorder, 1));
    p.drawLine(r.left(), r.bottom(), r.right(), r.bottom());

    // Target ~80px between major ticks
    double niceStep = 1.0;
    const double steps[] = {0.1, 0.25, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300};
    for (double s : steps) {
        if (s * m_pxPerSec >= 60) { niceStep = s; break; }
    }
    if (niceStep < 0.1) niceStep = 1.0;

    p.setFont(QFont("Segoe UI", 7));

    // Duration in seconds visible + some buffer
    double startSec = (m_scrollX) / m_pxPerSec;
    double endSec   = startSec + (r.width()) / m_pxPerSec + niceStep;

    double cur = floor(startSec / niceStep) * niceStep;
    while (cur <= endSec) {
        int x = kHeaderW + int((cur - startSec) * m_pxPerSec) - 0;
        if (x >= r.left() && x <= r.right()) {
            p.setPen(QPen(QColor(0x30, 0x30, 0x55), 1));
            p.drawLine(x, r.bottom() - 8, x, r.bottom());

            // Format timecode
            int totalMs = int(cur * 1000);
            int mm = totalMs / 60000;
            int ss = (totalMs % 60000) / 1000;
            QString label = (cur < 60.0)
                ? QString("%1:%2").arg(mm, 1).arg(ss, 2, 10, QChar('0'))
                : QString("%1:%2").arg(mm).arg(ss, 2, 10, QChar('0'));

            p.setPen(kTextLo);
            p.drawText(x + 3, r.top(), 60, r.height(), Qt::AlignVCenter | Qt::AlignLeft, label);
        }
        cur += niceStep;
    }
}

// ── Track header ─────────────────────────────────────────────────────────────
void TimelineWidget::paintTrackHeader(QPainter& p, const QRect& r, const QString& name, bool isVideo) {
    QLinearGradient bg(r.left(), 0, r.right(), 0);
    bg.setColorAt(0.0, QColor(0x12, 0x12, 0x1e));
    bg.setColorAt(1.0, QColor(0x10, 0x10, 0x18));
    p.fillRect(r, bg);

    QColor accent = isVideo ? kAccent : kClipAudioB;
    p.fillRect(r.left(), r.top() + 6, 2, r.height() - 12, accent);

    p.setFont(QFont("Segoe UI", 8, QFont::Bold));
    p.setPen(isVideo ? QColor(0x90, 0x88, 0xee) : QColor(0x60, 0xcc, 0xb8));
    p.drawText(r.adjusted(8, 0, 0, 0), Qt::AlignVCenter | Qt::AlignLeft, name);
}

// ── Track lane ───────────────────────────────────────────────────────────────
void TimelineWidget::paintTrackLane(QPainter& p, const QRect& r, bool isVideo) {
    QColor base = isVideo ? QColor(0x12, 0x12, 0x1c) : QColor(0x0f, 0x0f, 0x18);
    p.fillRect(r, base);
    int midY = r.top() + r.height() / 2;
    p.setPen(QPen(QColor(0x18, 0x18, 0x28), 1));
    p.drawLine(r.left(), midY, r.right(), midY);
}

// ── Clip ─────────────────────────────────────────────────────────────────────
void TimelineWidget::paintClip(QPainter& p, const QRect& r, const QString& label,
                                bool isVideo, bool selected, bool /*dragging*/) {
    if (r.width() < 4) return;

    QColor colA = isVideo ? kClipVideoA : kClipAudioA;
    QColor colB = isVideo ? kClipVideoB : kClipAudioB;

    QLinearGradient grad(r.left(), r.top(), r.left(), r.bottom());
    grad.setColorAt(0.0, colB);
    grad.setColorAt(0.5, colA);
    grad.setColorAt(1.0, colA.darker(130));

    QPainterPath path;
    path.addRoundedRect(r, 4, 4);
    p.fillPath(path, grad);

    // Top highlight
    QRect hl(r.left(), r.top(), r.width(), 3);
    QPainterPath hlPath;
    hlPath.addRoundedRect(hl, 4, 4);
    p.fillPath(hlPath, QColor(255, 255, 255, 30));

    // Border — yellow when selected
    if (selected) {
        p.setPen(QPen(kSelected, 2));
    } else {
        p.setPen(QPen(colB.lighter(140), 1));
    }
    p.drawPath(path);

    // Label
    if (r.width() > 30) {
        p.setFont(QFont("Segoe UI", 8));
        p.setPen(QColor(255, 255, 255, 200));
        p.drawText(r.adjusted(6, 0, -4, 0), Qt::AlignVCenter | Qt::AlignLeft,
                   QFontMetrics(p.font()).elidedText(label, Qt::ElideRight, r.width() - 10));
    }

    // Trim handles on edges
    QColor handleColor(255, 255, 255, 60);
    p.fillRect(r.left(), r.top(), 4, r.height(), handleColor);
    p.fillRect(r.right() - 4, r.top(), 4, r.height(), handleColor);
}

// ── Playhead ─────────────────────────────────────────────────────────────────
void TimelineWidget::paintPlayhead(QPainter& p, int x, int height) {
    if (x < kHeaderW) return;
    QPolygon head;
    head << QPoint(x - 6, 0) << QPoint(x + 6, 0)
         << QPoint(x + 6, kRulerH - 6)
         << QPoint(x, kRulerH)
         << QPoint(x - 6, kRulerH - 6);
    p.setPen(Qt::NoPen);
    p.setBrush(kAccent);
    p.drawPolygon(head);

    p.setPen(QPen(QColor(0x7c, 0x6b, 0xff, 200), 1));
    p.drawLine(x, kRulerH, x, height);
    p.setPen(QPen(QColor(0x7c, 0x6b, 0xff, 40), 3));
    p.drawLine(x, kRulerH, x, height);
}

// ── Drop indicator ────────────────────────────────────────────────────────────
void TimelineWidget::paintDropIndicator(QPainter& p, int x, int trackY, int trackH) {
    p.setPen(QPen(kDropLine, 2));
    p.drawLine(x, trackY, x, trackY + trackH);

    // Small arrow head at top
    p.setBrush(kDropLine);
    p.setPen(Qt::NoPen);
    QPolygon arrow;
    arrow << QPoint(x, trackY + 8) << QPoint(x - 5, trackY) << QPoint(x + 5, trackY);
    p.drawPolygon(arrow);
}

// ── Mouse ─────────────────────────────────────────────────────────────────────
void TimelineWidget::mousePressEvent(QMouseEvent* e) {
    const QPoint pos = e->position().toPoint();

    // Ruler click → move playhead
    if (pos.y() <= kRulerH && pos.x() > kHeaderW) {
        m_draggingPlayhead = true;
        qint64 ms = qMax(0LL, xToMs(pos.x()));
        m_playheadMs = ms;
        emit playheadMoved(ms);
        update();
        return;
    }

    // Clip hit test
    auto hit = hitTest(pos);
    if (hit.trackIdx >= 0 && !hit.clipId.isEmpty()) {
        m_selectedClipId = hit.clipId;
        emit clipSelected(hit.clipId);

        // Start drag
        m_draggingClip    = true;
        m_dragClipId      = hit.clipId;
        m_dragTrackIdx    = hit.trackIdx;

        // Compute offset
        if (m_timeline) {
            for (const auto& clip : m_timeline->track(hit.trackIdx)->clips()) {
                if (clip.id == hit.clipId) {
                    m_dragClipOrigMs = clip.timelineStartMs;
                    m_dragOffsetPx   = pos.x() - msToX(clip.timelineStartMs);
                    break;
                }
            }
        }
    } else {
        m_selectedClipId.clear();
    }
    update();
}

void TimelineWidget::mouseMoveEvent(QMouseEvent* e) {
    m_hoverX = e->position().x();
    const QPoint pos = e->position().toPoint();

    if (m_draggingPlayhead) {
        qint64 ms = qMax(0LL, xToMs(pos.x()));
        m_playheadMs = ms;
        emit playheadMoved(ms);
        update();
        return;
    }

    if (m_draggingClip && m_timeline) {
        int newX    = pos.x() - m_dragOffsetPx;
        qint64 newMs = qMax(0LL, xToMs(newX + m_dragOffsetPx - (msToX(0) - kHeaderW)));
        // Recalculate cleanly
        newMs = qMax(0LL, qint64((newX - kHeaderW + m_scrollX) * 1000.0 / m_pxPerSec));

        // Move clip in track
        auto* track = m_timeline->track(m_dragTrackIdx);
        if (track) {
            for (auto& clip : const_cast<QList<VidCut::Clip>&>(track->clips())) {
                if (clip.id == m_dragClipId) {
                    clip.timelineStartMs = newMs;
                    break;
                }
            }
        }
        update();
    }

    // Change cursor near clip edges
    auto hit = hitTest(pos);
    if (hit.trackIdx >= 0 && m_timeline) {
        for (const auto& clip : m_timeline->track(hit.trackIdx)->clips()) {
            if (clip.id == hit.clipId) {
                QRect r = clipRect(hit.trackIdx, clip);
                if (pos.x() <= r.left() + 6 || pos.x() >= r.right() - 6)
                    setCursor(Qt::SizeHorCursor);
                else
                    setCursor(Qt::OpenHandCursor);
                return;
            }
        }
    }
    setCursor(Qt::ArrowCursor);
}

void TimelineWidget::mouseReleaseEvent(QMouseEvent*) {
    m_draggingPlayhead = false;
    m_draggingClip     = false;
    m_dragClipId.clear();
    setCursor(Qt::ArrowCursor);
    update();
}

void TimelineWidget::leaveEvent(QEvent*) {
    m_hoverX = -1;
    update();
}

void TimelineWidget::resizeEvent(QResizeEvent* e) {
    QWidget::resizeEvent(e);
    update();
}

// ── Wheel zoom ────────────────────────────────────────────────────────────────
void TimelineWidget::wheelEvent(QWheelEvent* e) {
    if (e->modifiers() & Qt::ControlModifier) {
        double delta = e->angleDelta().y() > 0 ? 1.15 : (1.0/1.15);
        m_pxPerSec = qBound((double)kMinPxPerSec, m_pxPerSec * delta, (double)kMaxPxPerSec);
        update();
    } else {
        // Horizontal scroll
        m_scrollX = qMax(0, m_scrollX - e->angleDelta().y() / 2);
        update();
    }
}

// ── Drag & Drop (from Media Browser) ─────────────────────────────────────────
void TimelineWidget::dragEnterEvent(QDragEnterEvent* e) {
    if (e->mimeData()->hasFormat("application/x-vidcut-asset-id"))
        e->acceptProposedAction();
}

void TimelineWidget::dragMoveEvent(QDragMoveEvent* e) {
    if (!e->mimeData()->hasFormat("application/x-vidcut-asset-id")) return;
    e->acceptProposedAction();
    QPoint pos = e->position().toPoint();
    m_dropX = msToX(qMax(0LL, xToMs(pos.x())));
    m_dropTrackIdx = trackIndexAtY(pos.y());
    m_dropActive = true;
    update();
}

void TimelineWidget::dropEvent(QDropEvent* e) {
    m_dropActive = false;
    if (!e->mimeData()->hasFormat("application/x-vidcut-asset-id")) return;
    e->acceptProposedAction();

    QString assetId = QString::fromUtf8(e->mimeData()->data("application/x-vidcut-asset-id"));
    QPoint pos = e->position().toPoint();
    qint64 ms = qMax(0LL, xToMs(pos.x()));
    int trackIdx = trackIndexAtY(pos.y());
    if (trackIdx < 0) trackIdx = 0;

    emit requestAddClipToTrack(assetId, trackIdx, ms);
    update();
}
