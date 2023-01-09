#include <iostream>

extern "C" {
  void ocipkg_hello_world() {
    std::cout << "Hello from C++!" << std::endl;
  }
}
