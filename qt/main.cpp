#include "mainwindow.h"

#include <QApplication>

int main(int argc, char *argv[]) {
    QApplication a(argc, argv);
    MainWindow w;
    w.show();
    return a.exec();
}

extern "C" void *cpp_alloc(std::size_t size, std::align_val_t al) {
    return operator new(size, al);
}

extern "C" void cpp_free(void *ptr, std::align_val_t al) {
    operator delete(ptr, al);
}

extern "C" void rust_eh_personality() {
    abort();
}