#include "InspectorWidget.h"
#include <QLabel>
#include <QVBoxLayout>

InspectorWidget::InspectorWidget(QWidget* parent) : QWidget(parent) {
    setObjectName("inspectorWidget");
    setMinimumWidth(200);

    auto* layout = new QVBoxLayout(this);
    auto* label = new QLabel("No selection", this);
    label->setAlignment(Qt::AlignCenter);
    label->setObjectName("inspectorPlaceholder");
    layout->addWidget(label);
    layout->addStretch();
}
