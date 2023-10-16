#include "hyperparameter.h"
#include <functional>
#include <iostream>
#include <sstream>
#include <tuple>
#include <vector>

#define SAFE_BLOCK(block)                                                      \
  do {                                                                         \
    block                                                                      \
  } while (0)

#define ASSERT(test, message)                                                  \
  SAFE_BLOCK(if (!(test)) {                                                    \
    auto ss = std::stringstream();                                             \
    ss << #test << " failed: \n\t" << __FILE__ << ":" << __LINE__ << ":"       \
       << message << std::endl;                                                \
    throw ss.str();                                                            \
  })

using ttype = std::function<void(void)>;
using tentry = std::tuple<std::string, ttype>;
struct TestRunner {
  std::vector<tentry> tests_;
  TestRunner(std::initializer_list<tentry> tests) : tests_(tests) {}

  void operator()() {
    for (tentry e : tests_) {
      std::string name = std::get<0>(e);
      ttype test = std::get<1>(e);

      printf("%s...", name.c_str());
      try {
        test();
        printf("\tPASS\n");
      } catch (std::string err) {
        printf("\tFAILED\n");
        printf("\t%s\n", err.c_str());
      }
    }
  }
};

int main() {

  TestRunner runner = TestRunner({
      {"test xxhash",
       []() {
         ASSERT(5308235351123835395 ==
                    hyperparameter::xxh64::hash(
                        "0123456789abcdefghijklmnopqrstuvwxyz", 36, 42),
                "xxhash not match");
       }},
      {"test param scope create",
       []() {
         auto hp = hyperparameter::create();
         ASSERT(hp != NULL, "error create hyperparameter");
       }},

      {"test param scope default value for undefined",
       []() {
         auto hp = hyperparameter::create_shared();
         ASSERT(1 == hp->get((uint64_t)1, 1), "default value is expected");
       }},
      {"test param scope put parameter",
       []() {
         auto hp = hyperparameter::create_shared();
         hp->put("a", 1);
         hp->put("a.b", 2.0);
         hp->put("a.b.c", true);
         hp->put("a.b.c.d", "str");

         ASSERT(1 == hp->get("a", 0), "get int value");
         ASSERT(2.0 == hp->get("a.b", 0.0), "get float value");
         ASSERT(true == hp->get("a.b.c", false), "get bool value");
         ASSERT(std::string("str") == hp->get("a.b.c.d", ""), "get str value");
       }},
      {"test WITH_PARAMS",
       []() {
         ASSERT(0 == GET_PARAM(a.b, 1), "get undefined param");
         {
            auto aaa = WITH_PARAMS(a, 1);
           auto guard = WITH_PARAMS(a, 1, a.b, 2.0);
         }
       }},
  });
  runner();

  {
    auto aa = WITH_PARAMS(a.b, false);
    auto bb = WITH_PARAMS(A.B.C, "abc");

    std::cout << "\n:: (opt api) test param_scope enter" << std::endl
              << "expected: abc" << std::endl
              << "returned: " << GET_PARAM(A.B.C, "123") << std::endl
              << "expected: 0" << std::endl
              << "returned: " << GET_PARAM(a.b, 1) << std::endl
              << "expected: false" << std::endl
              << "returned: " << GET_PARAM(a.b, "true") << std::endl;
  }
  std::cout << "\n:: (opt api) test param_scope exit" << std::endl
            << "expected: 123" << std::endl
            << "returned: " << GET_PARAM(A.B.C, "123") << std::endl
            << "expected: 1" << std::endl
            << "returned: " << GET_PARAM(a.b, 1) << std::endl
            << "expected: true" << std::endl
            << "returned: " << GET_PARAM(a.b, "true") << std::endl;

  std::cout << "\n:: (opt api) test undefined" << std::endl
            << "expected: 100" << std::endl
            << "returned: " << GET_PARAM(d.e.f, 100) << std::endl;

  std::cout << "in main" << std::endl;

  std::cout << "test1.test2: " << GET_PARAM(test1.test2, 100) << std::endl;

  // ===== bool test ====
  std::cout << "\n:: test bool parameter" << std::endl
            << "expected: true" << std::endl
            << "returned: " << GET_PARAM(test1.bool1, false) << std::endl
            << "expected: true" << std::endl
            << "returned: " << GET_PARAM(test1.bool2, false) << std::endl
            << "expected: false" << std::endl
            << "returned: " << GET_PARAM(test1.bool3, true) << std::endl
            << "expected: false" << std::endl
            << "returned: " << GET_PARAM(test1.bool4, true) << std::endl;
  return 0;
}