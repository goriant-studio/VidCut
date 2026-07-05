#include "ui/MainWindow.h"
#include <QApplication>
#include <QFont>
#include <QFontDatabase>

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    app.setApplicationName("VidCut");
    app.setApplicationVersion("0.1.0");
    app.setOrganizationName("Goriant Studio");

    // Use Inter if available, fall back to Segoe UI
    QFontDatabase::addApplicationFont(":/fonts/Inter-Regular.ttf");
    app.setFont(QFont("Inter", 10));

    MainWindow window;
    window.show();

    return app.exec();
}
