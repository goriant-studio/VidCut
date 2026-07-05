#pragma once
#include <QWidget>

// Shows properties of the selected clip or effect.
// Phase 1: empty placeholder.
class InspectorWidget : public QWidget {
    Q_OBJECT
public:
    explicit InspectorWidget(QWidget* parent = nullptr);
};
