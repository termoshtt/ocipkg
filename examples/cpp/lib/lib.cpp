#include <iostream>

extern "C" {
  void hello_from_cpp() {
    std::cout << "Hello from C++!" << std::endl;
  }
}
