#include "ExportDialog.h"

#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QFormLayout>
#include <QGroupBox>
#include <QFileDialog>
#include <QStandardPaths>
#include <QMessageBox>
#include <QDesktopServices>
#include <QUrl>
#include <QFileInfo>

ExportDialog::ExportDialog(VidCut::Timeline* timeline, QWidget* parent)
    : QDialog(parent), m_timeline(timeline)
{
    setObjectName("exportDialog");
    setWindowTitle("Export Video");
    setMinimumWidth(480);
    setModal(true);
    setupUI();

    m_encoder = new VidCut::ExportEncoder(this);
    connect(m_encoder, &VidCut::ExportEncoder::progressChanged,
            this,      &ExportDialog::onProgress);
    connect(m_encoder, &VidCut::ExportEncoder::exportFinished,
            this,      &ExportDialog::onFinished);
}

void ExportDialog::setupUI() {
    auto* mainLayout = new QVBoxLayout(this);
    mainLayout->setSpacing(12);
    mainLayout->setContentsMargins(16, 16, 16, 16);

    // ── Output path ──────────────────────────────────────────────────────────
    auto* pathGroup = new QGroupBox("Output File", this);
    auto* pathLayout = new QHBoxLayout(pathGroup);
    m_pathEdit = new QLineEdit(this);
    m_pathEdit->setPlaceholderText("Choose output file path...");

    QString defaultPath = QStandardPaths::writableLocation(QStandardPaths::MoviesLocation)
                          + "/VidCut_Export.mp4";
    m_pathEdit->setText(defaultPath);

    auto* browseBtn = new QPushButton("Browse…", this);
    browseBtn->setObjectName("browseBtn");
    connect(browseBtn, &QPushButton::clicked, this, &ExportDialog::onBrowse);
    pathLayout->addWidget(m_pathEdit, 1);
    pathLayout->addWidget(browseBtn);
    mainLayout->addWidget(pathGroup);

    // ── Settings ──────────────────────────────────────────────────────────────
    auto* settingsGroup = new QGroupBox("Export Settings", this);
    auto* form = new QFormLayout(settingsGroup);

    m_resCombo = new QComboBox(this);
    m_resCombo->addItem("1920 × 1080  (1080p)", QSize(1920, 1080));
    m_resCombo->addItem("3840 × 2160  (4K)",    QSize(3840, 2160));
    m_resCombo->addItem("1280 × 720   (720p)",  QSize(1280, 720));
    m_resCombo->addItem("854 × 480    (480p)",  QSize(854, 480));
    form->addRow("Resolution:", m_resCombo);

    m_fpsSpinBox = new QSpinBox(this);
    m_fpsSpinBox->setRange(1, 120);
    m_fpsSpinBox->setValue(30);
    m_fpsSpinBox->setSuffix(" fps");
    form->addRow("Frame Rate:", m_fpsSpinBox);

    m_bitrateSpinBox = new QSpinBox(this);
    m_bitrateSpinBox->setRange(500, 50000);
    m_bitrateSpinBox->setValue(8000);
    m_bitrateSpinBox->setSuffix(" kbps");
    m_bitrateSpinBox->setSingleStep(500);
    form->addRow("Video Bitrate:", m_bitrateSpinBox);

    mainLayout->addWidget(settingsGroup);

    // ── Progress ──────────────────────────────────────────────────────────────
    m_progress = new QProgressBar(this);
    m_progress->setRange(0, 100);
    m_progress->setValue(0);
    m_progress->setTextVisible(true);
    m_progress->setVisible(false);
    mainLayout->addWidget(m_progress);

    m_statusLabel = new QLabel(this);
    m_statusLabel->setObjectName("exportStatusLabel");
    m_statusLabel->setAlignment(Qt::AlignCenter);
    m_statusLabel->setVisible(false);
    mainLayout->addWidget(m_statusLabel);

    // ── Buttons ───────────────────────────────────────────────────────────────
    auto* btnRow = new QHBoxLayout();
    m_cancelBtn = new QPushButton("Cancel", this);
    m_exportBtn = new QPushButton("Export", this);
    m_exportBtn->setObjectName("exportBtn");
    m_exportBtn->setDefault(true);

    btnRow->addStretch();
    btnRow->addWidget(m_cancelBtn);
    btnRow->addWidget(m_exportBtn);
    mainLayout->addLayout(btnRow);

    connect(m_exportBtn, &QPushButton::clicked, this, &ExportDialog::onExport);
    connect(m_cancelBtn, &QPushButton::clicked, this, &ExportDialog::onCancel);
}

void ExportDialog::onBrowse() {
    QString path = QFileDialog::getSaveFileName(
        this,
        "Export Video As…",
        m_pathEdit->text(),
        "MP4 Video (*.mp4);;All Files (*)"
    );
    if (!path.isEmpty()) m_pathEdit->setText(path);
}

void ExportDialog::onExport() {
    if (m_exporting) return;
    QString path = m_pathEdit->text().trimmed();
    if (path.isEmpty()) {
        QMessageBox::warning(this, "Export", "Please choose an output file path.");
        return;
    }

    QSize res = m_resCombo->currentData().toSize();
    VidCut::ExportSettings settings;
    settings.outputPath   = path;
    settings.width        = res.width();
    settings.height       = res.height();
    settings.fps          = m_fpsSpinBox->value();
    settings.videoBitrate = m_bitrateSpinBox->value() * 1000;
    settings.audioBitrate = 192000;

    m_exporting = true;
    m_exportBtn->setEnabled(false);
    m_cancelBtn->setText("Stop");
    m_progress->setVisible(true);
    m_progress->setValue(0);
    m_statusLabel->setVisible(true);
    m_statusLabel->setText("Exporting…");

    m_encoder->startExport(settings, m_timeline);
}

void ExportDialog::onCancel() {
    if (m_exporting) {
        m_encoder->cancel();
    } else {
        reject();
    }
}

void ExportDialog::onProgress(int percent) {
    m_progress->setValue(percent);
    m_statusLabel->setText(QString("Exporting… %1%").arg(percent));
}

void ExportDialog::onFinished(bool success, const QString& path) {
    m_exporting = false;
    m_exportBtn->setEnabled(true);
    m_cancelBtn->setText("Close");

    if (success) {
        m_progress->setValue(100);
        m_statusLabel->setText("✅ Export complete!");
        auto* openBtn = new QPushButton("Open in Explorer", this);
        connect(openBtn, &QPushButton::clicked, this, [path]{
            QDesktopServices::openUrl(QUrl::fromLocalFile(QFileInfo(path).absolutePath()));
        });
        layout()->addWidget(openBtn);
        m_exportBtn->setVisible(false);
    } else {
        m_statusLabel->setText("❌ Export failed. Check that output path is writable.");
    }
}
