#include "MediaBrowserWidget.h"
#include "MediaPoolSignals.h"
#include "../media/ThumbnailGenerator.h"

#include <QFileDialog>
#include <QStandardPaths>
#include <QListWidgetItem>
#include <QMimeData>
#include <QDrag>
#include <QPixmap>
#include <QPainter>
#include <QFontMetrics>
#include <QApplication>
#include <QUrl>
#include <QFileInfo>
#include <QTimer>

// MIME type used when dragging an asset from the browser to the timeline
static const QString kAssetMime = "application/x-vidcut-asset-id";

MediaBrowserWidget::MediaBrowserWidget(QWidget* parent) : QWidget(parent) {
    setObjectName("mediaBrowserWidget");
    setMinimumWidth(220);
    setAcceptDrops(true);

    // -- Layout
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(4, 4, 4, 4);
    layout->setSpacing(4);

    // Import button
    m_importBtn = new QPushButton("＋  Import Media", this);
    m_importBtn->setObjectName("importBtn");
    m_importBtn->setCursor(Qt::PointingHandCursor);
    layout->addWidget(m_importBtn);

    // List
    m_list = new QListWidget(this);
    m_list->setObjectName("mediaBrowserList");
    m_list->setViewMode(QListWidget::IconMode);
    m_list->setIconSize(QSize(140, 80));
    m_list->setGridSize(QSize(150, 110));
    m_list->setResizeMode(QListWidget::Adjust);
    m_list->setMovement(QListWidget::Static);
    m_list->setDragEnabled(true);
    m_list->setSelectionMode(QAbstractItemView::ExtendedSelection);
    m_list->setSpacing(4);
    layout->addWidget(m_list, 1);

    // Drop hint label
    m_dropHint = new QLabel("Drop video files here\nor click ＋ Import", this);
    m_dropHint->setObjectName("dropHintLabel");
    m_dropHint->setAlignment(Qt::AlignCenter);
    m_dropHint->setWordWrap(true);
    layout->addWidget(m_dropHint);

    // Connections
    connect(m_importBtn, &QPushButton::clicked, this, &MediaBrowserWidget::onImportClicked);
    connect(m_list, &QListWidget::itemDoubleClicked, this, &MediaBrowserWidget::onItemDoubleClicked);

    // Enable drag from list
    connect(m_list, &QListWidget::itemPressed, this, [this](QListWidgetItem* item) {
        if (QApplication::mouseButtons() & Qt::LeftButton) {
            startDragForItem(item);
        }
    });
}

void MediaBrowserWidget::setMediaPool(VidCut::MediaPool* pool) {
    m_pool = pool;
    // Add any pre-existing assets
    for (auto* a : m_pool->assets())
        addItemForAsset(a);
}

void MediaBrowserWidget::onImportClicked() {
    QStringList paths = QFileDialog::getOpenFileNames(
        this,
        tr("Import Media"),
        QStandardPaths::writableLocation(QStandardPaths::MoviesLocation),
        tr("Video Files (*.mp4 *.mov *.mkv *.avi *.wmv *.webm *.m4v *.ts *.mts *.m2ts);;"
           "Audio Files (*.mp3 *.wav *.aac *.flac *.ogg *.m4a);;"
           "All Files (*)")
    );
    importFiles(paths);
}

void MediaBrowserWidget::importFiles(const QStringList& paths) {
    if (!m_pool || paths.isEmpty()) return;
    for (const QString& p : paths) {
        auto* asset = m_pool->importFile(p);
        if (asset) onAssetAdded(asset);
    }
}

void MediaBrowserWidget::onAssetAdded(VidCut::MediaAsset* asset) {
    addItemForAsset(asset);
    m_dropHint->setVisible(m_list->count() == 0);
}

void MediaBrowserWidget::addItemForAsset(VidCut::MediaAsset* asset) {
    auto* item = new QListWidgetItem(m_list);
    item->setData(Qt::UserRole, asset->id);

    // File name (elided)
    QFontMetrics fm(m_list->font());
    item->setText(fm.elidedText(asset->name, Qt::ElideMiddle, 138));
    item->setToolTip(asset->filePath);

    // Placeholder thumbnail (will be replaced when ready)
    QPixmap thumb(140, 80);
    thumb.fill(QColor(0x18, 0x18, 0x28));
    {
        QPainter p(&thumb);
        p.setPen(QColor(0x50, 0x50, 0x70));
        p.setFont(QFont("Segoe UI", 8));
        p.drawText(thumb.rect(), Qt::AlignCenter, asset->hasVideo ? "⏳ loading..." : "🎵");
    }
    item->setIcon(QIcon(thumb));

    m_dropHint->setVisible(false);

    // Request real thumbnail async
    if (!m_pool) return;
    auto* thumbGen = new VidCut::ThumbnailGenerator(this);
    connect(thumbGen, &VidCut::ThumbnailGenerator::thumbnailReady,
        this, [this, thumbGen](const QString& fp, qint64, const QImage& img) {
            // Find item by filePath
            if (!img.isNull()) {
                for (int i = 0; i < m_list->count(); ++i) {
                    auto* itm = m_list->item(i);
                    // match by checking pool
                    if (m_pool) {
                        for (auto* a : m_pool->assets()) {
                            if (a->filePath == fp && a->id == itm->data(Qt::UserRole).toString()) {
                                QPixmap px = QPixmap::fromImage(img);
                                itm->setIcon(QIcon(px));
                                break;
                            }
                        }
                    }
                }
            }
            thumbGen->deleteLater();
        });

    thumbGen->requestThumbnail(asset->filePath, 1000);
}

void MediaBrowserWidget::onItemDoubleClicked(QListWidgetItem* item) {
    if (!m_pool) return;
    QString id = item->data(Qt::UserRole).toString();
    auto* asset = m_pool->findById(id);
    if (asset) emit assetDoubleClicked(asset);
}

void MediaBrowserWidget::startDragForItem(QListWidgetItem* item) {
    if (!item) return;
    QString id = item->data(Qt::UserRole).toString();
    if (id.isEmpty()) return;

    auto* drag = new QDrag(this);
    auto* mime = new QMimeData();
    mime->setData(kAssetMime, id.toUtf8());
    drag->setMimeData(mime);

    // Drag pixmap = item icon scaled
    QPixmap px = item->icon().pixmap(120, 68);
    drag->setPixmap(px);
    drag->setHotSpot(px.rect().center());

    drag->exec(Qt::CopyAction);
}

// ── File drop from OS ──────────────────────────────────────────

void MediaBrowserWidget::dragEnterEvent(QDragEnterEvent* e) {
    if (e->mimeData()->hasUrls()) e->acceptProposedAction();
}

void MediaBrowserWidget::dropEvent(QDropEvent* e) {
    QStringList paths;
    for (const QUrl& url : e->mimeData()->urls()) {
        if (url.isLocalFile()) paths << url.toLocalFile();
    }
    importFiles(paths);
    e->acceptProposedAction();
}
