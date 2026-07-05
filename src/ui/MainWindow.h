#pragma once
#include <QMainWindow>

#include "vidcut/MediaPool.h"
#include "vidcut/Timeline.h"
#include "vidcut/Track.h"
#include "vidcut/CommandManager.h"

class TimelineWidget;
class PreviewWidget;
class MediaBrowserWidget;
class InspectorWidget;
class ToolBar;

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow() override = default;

private slots:
    void onImportMedia();
    void onExport();
    void onUndo();
    void onRedo();
    void onAddClipToTrack(const QString& assetId, int trackIndex, qint64 timelineMs);
    void onClipSelected(const QString& clipId);

private:
    void setupUI();
    void setupMenuBar();
    void setupToolbarActions();
    void loadStylesheet();
    void ensureDefaultTracks();

    // Data
    VidCut::MediaPool      m_mediaPool;   // value — not a QObject
    VidCut::Timeline*      m_timeline   = nullptr;
    VidCut::CommandManager m_cmdMgr;

    // UI
    TimelineWidget*      m_timeline_w   = nullptr;
    PreviewWidget*       m_preview      = nullptr;
    MediaBrowserWidget*  m_mediaBrowser = nullptr;
    InspectorWidget*     m_inspector    = nullptr;
    ToolBar*             m_toolBar      = nullptr;
};
