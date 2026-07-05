#pragma once
#include <QDialog>
#include <QLineEdit>
#include <QSpinBox>
#include <QComboBox>
#include <QProgressBar>
#include <QPushButton>
#include <QLabel>

#include "vidcut/Timeline.h"
#include "../media/ExportEncoder.h"

class ExportDialog : public QDialog {
    Q_OBJECT
public:
    explicit ExportDialog(VidCut::Timeline* timeline, QWidget* parent = nullptr);

private slots:
    void onBrowse();
    void onExport();
    void onCancel();
    void onProgress(int percent);
    void onFinished(bool success, const QString& path);

private:
    void setupUI();

    VidCut::Timeline*        m_timeline = nullptr;
    VidCut::ExportEncoder*   m_encoder  = nullptr;

    QLineEdit*   m_pathEdit      = nullptr;
    QComboBox*   m_resCombo      = nullptr;
    QSpinBox*    m_fpsSpinBox    = nullptr;
    QSpinBox*    m_bitrateSpinBox= nullptr;
    QProgressBar* m_progress     = nullptr;
    QPushButton* m_exportBtn     = nullptr;
    QPushButton* m_cancelBtn     = nullptr;
    QLabel*      m_statusLabel   = nullptr;

    bool m_exporting = false;
};
