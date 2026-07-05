#pragma once
#include <QWidget>
#include <QListWidget>
#include <QPushButton>
#include <QVBoxLayout>
#include <QLabel>
#include <QDragEnterEvent>
#include <QDropEvent>

#include "vidcut/MediaPool.h"
#include "vidcut/MediaAsset.h"

class MediaBrowserWidget : public QWidget {
    Q_OBJECT
public:
    explicit MediaBrowserWidget(QWidget* parent = nullptr);

    void setMediaPool(VidCut::MediaPool* pool);
    VidCut::MediaPool* mediaPool() const { return m_pool; }

signals:
    void assetDoubleClicked(VidCut::MediaAsset* asset);
    void assetDragStarted(VidCut::MediaAsset* asset);

protected:
    void dragEnterEvent(QDragEnterEvent* e) override;
    void dropEvent(QDropEvent* e) override;

private slots:
    void onImportClicked();
    void onAssetAdded(VidCut::MediaAsset* asset);
    void onItemDoubleClicked(QListWidgetItem* item);

private:
    void addItemForAsset(VidCut::MediaAsset* asset);
    void importFiles(const QStringList& paths);
    void startDragForItem(QListWidgetItem* item);

    VidCut::MediaPool*  m_pool = nullptr;
    QListWidget*        m_list = nullptr;
    QPushButton*        m_importBtn = nullptr;
    QLabel*             m_dropHint = nullptr;
};
