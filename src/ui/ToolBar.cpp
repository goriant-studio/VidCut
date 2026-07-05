#include "ToolBar.h"
#include <QAction>

ToolBar::ToolBar(QWidget* parent) : QToolBar(parent) {
    setObjectName("mainToolBar");
    setMovable(false);
    setIconSize(QSize(20, 20));

    addAction("⏮")->setToolTip("Go to Start");
    addAction("⏪")->setToolTip("Rewind");
    addAction("▶")->setToolTip("Play / Pause");
    addAction("⏩")->setToolTip("Fast Forward");
    addAction("⏭")->setToolTip("Go to End");
    addSeparator();
    addAction("✂")->setToolTip("Split Clip");
    addAction("🔗")->setToolTip("Link / Unlink");
    addSeparator();
    addAction("⚙")->setToolTip("Export");
}
