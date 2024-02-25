#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QKeyEvent>
#include "terncalc.h"

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    void keyPressEvent(QKeyEvent *event) override;
    ~MainWindow();

private slots:
    void button_clicked(char input);
    void about();

private:
    Ui::MainWindow *ui;
    Terncalc *calc;
    int64_t value;
    void display_ternary();
    void display_decimal();
    void update_enabled();
};
#endif // MAINWINDOW_H
