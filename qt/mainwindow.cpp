#include "mainwindow.h"
#include "ui_mainwindow.h"
#include <QMessageBox>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    calc = calc_new();
    value = 0;
    ui->setupUi(this);

    QObject::connect(ui->numberZero, &QPushButton::clicked, this, [=](){this->button_clicked('0');});
    QObject::connect(ui->numberOne, &QPushButton::clicked, this, [=](){this->button_clicked('1');});
    QObject::connect(ui->numberTwo, &QPushButton::clicked, this, [=](){this->button_clicked('2');});
    QObject::connect(ui->actionPlus, &QPushButton::clicked, this, [=](){this->button_clicked('+');});
    QObject::connect(ui->actionMinus, &QPushButton::clicked, this, [=](){this->button_clicked('-');});
    QObject::connect(ui->actionTimes, &QPushButton::clicked, this, [=](){this->button_clicked('*');});
    QObject::connect(ui->actionDiv, &QPushButton::clicked, this, [=](){this->button_clicked('/');});
    QObject::connect(ui->actionEquals, &QPushButton::clicked, this, [=](){this->button_clicked('=');});
    QObject::connect(ui->actionOpen, &QPushButton::clicked, this, [=](){this->button_clicked('(');});
    QObject::connect(ui->actionClose, &QPushButton::clicked, this, [=](){this->button_clicked(')');});
    QObject::connect(ui->actionClear, &QPushButton::clicked, this, [=](){this->button_clicked('C');});
    QObject::connect(ui->actionClearAll, &QPushButton::clicked, this, [=](){this->button_clicked('A');});
    QObject::connect(ui->actionClearSingle, &QPushButton::clicked, this, [=](){this->button_clicked('U');});
    QObject::connect(ui->actionCheat, &QPushButton::clicked, this, [=](){this->display_decimal();});
    QObject::connect(ui->actionUndo, &QAction::triggered, this, [=](){this->button_clicked('U');});
    QObject::connect(ui->actionRedo, &QAction::triggered, this, [=](){this->button_clicked('R');});

    QObject::connect(ui->actionQuit, &QAction::triggered, this, &MainWindow::close);
    QObject::connect(ui->actionAbout, &QAction::triggered, this, &MainWindow::about);

    update_enabled();
}

void MainWindow::keyPressEvent(QKeyEvent *event) {
    switch (event->key()) {
    case Qt::Key_0:
        ui->numberZero->animateClick();
        break;
    case Qt::Key_1:
        ui->numberOne->animateClick();
        break;
    case Qt::Key_2:
        ui->numberTwo->animateClick();
        break;
    case Qt::Key_Plus:
        ui->actionPlus->animateClick();
        break;
    case Qt::Key_Minus:
        ui->actionMinus->animateClick();
        break;
    case Qt::Key_Asterisk:
        ui->actionTimes->animateClick();
        break;
    case Qt::Key_Slash:
        ui->actionDiv->animateClick();
        break;
    case Qt::Key_ParenLeft:
        ui->actionOpen->animateClick();
        break;
    case Qt::Key_ParenRight:
        ui->actionClose->animateClick();
        break;
    case Qt::Key_C:
        ui->actionClear->animateClick();
        break;
    case Qt::Key_Backspace:
        ui->actionClearSingle->animateClick();
        break;
    case Qt::Key_Delete:
        ui->actionClearAll->animateClick();
        break;
    case Qt::Key_Return:
    case Qt::Key_Enter:
    case Qt::Key_Equal:
        ui->actionEquals->animateClick();
        break;
    default:
        QMainWindow::keyPressEvent(event);
    }
}

void set_enabled_or_tooltip(QWidget* widget, const char* text) {
    if (text == nullptr) {
        widget->setEnabled(true);
        widget->setToolTip("");
    } else {
        widget->setEnabled(false);
        widget->setToolTip(QString::fromUtf8(text));
    }
}
// Apparently QWidget and QAction do not have a common ancestor
// with "enabled" and "tooltip"?
void set_enabled_or_tooltip_action(QAction* action, const char* text) {
    if (text == nullptr) {
        action->setEnabled(true);
        action->setToolTip("");
    } else {
        action->setEnabled(false);
        action->setToolTip(QString::fromUtf8(text));
    }
}
void MainWindow::display_ternary() {
    char text[64];
    int pos = number_to_text(&text, value);
    ui->numberOutput->setText(QString::fromUtf8(&text[pos]));
}
void MainWindow::display_decimal() {
    ui->numberOutput->setText(QString::number(value));
}
void MainWindow::update_enabled() {
    const char* vector[14];
    calc_enabled(calc, &vector);

    set_enabled_or_tooltip(ui->numberZero, vector[0]);
    set_enabled_or_tooltip(ui->numberOne, vector[1]);
    set_enabled_or_tooltip(ui->numberTwo, vector[2]);
    set_enabled_or_tooltip(ui->actionPlus, vector[3]);
    set_enabled_or_tooltip(ui->actionMinus, vector[4]);
    set_enabled_or_tooltip(ui->actionTimes, vector[5]);
    set_enabled_or_tooltip(ui->actionDiv, vector[6]);
    set_enabled_or_tooltip(ui->actionOpen, vector[7]);
    set_enabled_or_tooltip(ui->actionClose, vector[8]);
    set_enabled_or_tooltip(ui->actionEquals, vector[9]);
    set_enabled_or_tooltip(ui->actionClear, vector[10]);
    set_enabled_or_tooltip(ui->actionClearAll, vector[11]);
    set_enabled_or_tooltip(ui->actionClearSingle, vector[12]);

    set_enabled_or_tooltip_action(ui->actionUndo, vector[12]);
    set_enabled_or_tooltip_action(ui->actionRedo, vector[13]);
}

MainWindow::~MainWindow()
{
    calc_drop(calc);
    delete ui;
}

void MainWindow::button_clicked(char input) {
    value = calc_input(calc, input);
    display_ternary();
    update_enabled();
}

void MainWindow::about() {
    QMessageBox::about(this, "Why", "Why not");
}
