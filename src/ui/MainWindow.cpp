#include "MainWindow.h"
#include "TimelineWidget.h"
#include "PreviewWidget.h"
#include "MediaBrowserWidget.h"
#include "InspectorWidget.h"
#include "ToolBar.h"
#include "ExportDialog.h"

#include "vidcut/Clip.h"
#include "vidcut/MediaAsset.h"

#include <QApplication>
#include <QDockWidget>
#include <QMenuBar>
#include <QMenu>
#include <QAction>
#include <QFile>
#include <QSplitter>
#include <QStatusBar>
#include <QKeySequence>
#include <QUuid>
#include <QFileDialog>
#include <QStandardPaths>
#include <QMessageBox>

MainWindow::MainWindow(QWidget* parent) : QMainWindow(parent) {
    setWindowTitle("VidCut");
    setMinimumSize(1280, 720);
    resize(1600, 900);

    // ── Data ─────────────────────────────────────────────────────────────────
    m_timeline  = new VidCut::Timeline();
    ensureDefaultTracks();

    setupUI();
    setupMenuBar();
    loadStylesheet();

    statusBar()->showMessage("Ready  —  Import media and drag clips to the timeline");
}

// ── Default tracks ────────────────────────────────────────────────────────────
void MainWindow::ensureDefaultTracks() {
    if (m_timeline->trackCount() == 0) {
        m_timeline->addTrack(new VidCut::Track(VidCut::TrackType::Video, "V2"));
        m_timeline->addTrack(new VidCut::Track(VidCut::TrackType::Video, "V1"));
        m_timeline->addTrack(new VidCut::Track(VidCut::TrackType::Audio, "A1"));
        m_timeline->addTrack(new VidCut::Track(VidCut::TrackType::Audio, "A2"));
    }
}

// ── UI Setup ──────────────────────────────────────────────────────────────────
void MainWindow::setupUI() {
    // Toolbar
    m_toolBar = new ToolBar(this);
    addToolBar(m_toolBar);

    // Preview + Timeline vertical splitter (center)
    m_preview    = new PreviewWidget(this);
    m_timeline_w = new TimelineWidget(this);
    m_timeline_w->setTimeline(m_timeline);
    m_timeline_w->setMediaPool(&m_mediaPool);

    auto* centerSplitter = new QSplitter(Qt::Vertical, this);
    centerSplitter->addWidget(m_preview);
    centerSplitter->addWidget(m_timeline_w);
    centerSplitter->setStretchFactor(0, 3);
    centerSplitter->setStretchFactor(1, 2);
    setCentralWidget(centerSplitter);

    // Media Browser dock (left)
    m_mediaBrowser = new MediaBrowserWidget(this);
    m_mediaBrowser->setMediaPool(&m_mediaPool);

    auto* mediaDock = new QDockWidget("Media Browser", this);
    mediaDock->setObjectName("mediaBrowserDock");
    mediaDock->setWidget(m_mediaBrowser);
    mediaDock->setFeatures(QDockWidget::DockWidgetMovable | QDockWidget::DockWidgetFloatable);
    addDockWidget(Qt::LeftDockWidgetArea, mediaDock);

    // Inspector dock (right)
    m_inspector = new InspectorWidget(this);
    auto* inspectorDock = new QDockWidget("Inspector", this);
    inspectorDock->setObjectName("inspectorDock");
    inspectorDock->setWidget(m_inspector);
    inspectorDock->setFeatures(QDockWidget::DockWidgetMovable | QDockWidget::DockWidgetFloatable);
    addDockWidget(Qt::RightDockWidgetArea, inspectorDock);

    // ── Signal Wiring ─────────────────────────────────────────────────────────

    // Timeline playhead ↔ Preview seek
    connect(m_timeline_w, &TimelineWidget::playheadMoved,
            m_preview,    &PreviewWidget::seekTo);
    connect(m_preview,    &PreviewWidget::positionChanged,
            m_timeline_w, &TimelineWidget::setPlayheadMs);

    // Double-click asset in browser → load in preview
    connect(m_mediaBrowser, &MediaBrowserWidget::assetDoubleClicked,
            m_preview,      &PreviewWidget::loadAsset);

    // Drop asset on timeline → add clip
    connect(m_timeline_w, &TimelineWidget::requestAddClipToTrack,
            this,          &MainWindow::onAddClipToTrack);

    // Clip selected → status bar
    connect(m_timeline_w, &TimelineWidget::clipSelected,
            this, &MainWindow::onClipSelected);
}

// ── Menu ──────────────────────────────────────────────────────────────────────
void MainWindow::setupMenuBar() {
    // File
    auto* fileMenu = menuBar()->addMenu("&File");
    fileMenu->addAction("Import Media…", QKeySequence("Ctrl+I"),
                        this, &MainWindow::onImportMedia);
    fileMenu->addSeparator();
    fileMenu->addAction("Export…", QKeySequence("Ctrl+E"),
                        this, &MainWindow::onExport);
    fileMenu->addSeparator();
    fileMenu->addAction("&Quit", QKeySequence::Quit, this, &QWidget::close);

    // Edit
    auto* editMenu = menuBar()->addMenu("&Edit");
    auto* undoAct = editMenu->addAction("Undo", QKeySequence::Undo,
                                         this, &MainWindow::onUndo);
    undoAct->setEnabled(false);
    auto* redoAct = editMenu->addAction("Redo", QKeySequence::Redo,
                                         this, &MainWindow::onRedo);
    redoAct->setEnabled(false);

    // View
    auto* viewMenu = menuBar()->addMenu("&View");
    viewMenu->addAction("Zoom In",    QKeySequence("Ctrl+="), m_timeline_w, &TimelineWidget::zoomIn);
    viewMenu->addAction("Zoom Out",   QKeySequence("Ctrl+-"), m_timeline_w, &TimelineWidget::zoomOut);
    viewMenu->addAction("Zoom Reset", QKeySequence("Ctrl+0"), m_timeline_w, &TimelineWidget::zoomReset);

    menuBar()->addMenu("&Help");
}

// ── Slots ──────────────────────────────────────────────────────────────────────
void MainWindow::onImportMedia() {
    // Route through MediaBrowserWidget so items appear in the browser UI
    QMetaObject::invokeMethod(m_mediaBrowser, "onImportClicked");

}

void MainWindow::onExport() {
    if (m_timeline->durationMs() == 0) {
        QMessageBox::information(this, "Export",
            "The timeline is empty. Add some clips before exporting.");
        return;
    }
    ExportDialog dlg(m_timeline, this);
    dlg.exec();
}

void MainWindow::onUndo() {
    m_cmdMgr.undo();
}

void MainWindow::onRedo() {
    m_cmdMgr.redo();
}

void MainWindow::onAddClipToTrack(const QString& assetId, int trackIndex, qint64 timelineMs) {
    auto* asset = m_mediaPool.findById(assetId);
    if (!asset) return;

    auto* track = m_timeline->track(trackIndex);
    if (!track) return;

    VidCut::Clip clip;
    clip.id             = QUuid::createUuid().toString(QUuid::WithoutBraces);
    clip.asset          = asset;
    clip.srcStartMs     = 0;
    clip.srcEndMs       = asset->durationMs;
    clip.timelineStartMs= timelineMs;

    track->addClip(clip);
    m_timeline_w->update();
    statusBar()->showMessage(QString("Added '%1' to %2").arg(asset->name, track->name()));
}

void MainWindow::onClipSelected(const QString& clipId) {
    statusBar()->showMessage(QString("Selected clip: %1").arg(clipId.left(8)));
}

// ── Stylesheet ────────────────────────────────────────────────────────────────
void MainWindow::loadStylesheet() {
    QFile f(":/styles/dark_theme.qss");
    if (f.open(QIODevice::ReadOnly)) {
        qApp->setStyleSheet(f.readAll());
    }
}
