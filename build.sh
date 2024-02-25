#!/bin/sh
cargo build
cd build
make clean
qmake6 ../qt/qt.pro CONFIG+=debug QMAKE_CXX=clang++ QMAKE_LINK=clang++
make
