/**
 * clang++ -Wall -L target/debug/ -lcert_get_core_ffi -o ffi-example ffi-example.cpp
 * LD_LIBRARY_PATH=target/debug ./ffi-example
 */

extern "C" {
    void say_hello();
}

int main() {
    say_hello();
}
