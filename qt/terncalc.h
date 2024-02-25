#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct Terncalc;

extern "C" {

Terncalc *calc_new();

void calc_drop(Terncalc *calc);

int64_t calc_input(Terncalc *calc, unsigned char input);

void calc_enabled(Terncalc *calc, const char *(*vector)[14]);

int number_to_text(char (*chars)[64], int64_t number);

extern "C" void *cpp_alloc(std::size_t size, std::align_val_t al);

extern "C" void cpp_free(void *ptr, std::align_val_t al);

} // extern "C"
