#include "hyperparameter.h"
#include <functional>
#include <iostream>
#include <sstream>
#include <tuple>
#include <vector>

#define SAFE_BLOCK(block) \
  do                      \
  {                       \
    block                 \
  } while (0)

#define ASSERT(test, message)                                            \
  SAFE_BLOCK(if (!(test)) {                                              \
    auto ss = std::stringstream();                                       \
    ss << #test << " failed: \n\t" << __FILE__ << ":" << __LINE__ << ":" \
       << message << std::endl;                                          \
    throw ss.str();                                                      \
  })

using ttype = std::function<void(void)>;
using tentry = std::tuple<std::string, ttype>;
struct TestRunner
{
  std::vector<tentry> tests_;
  TestRunner(std::initializer_list<tentry> tests) : tests_(tests) {}

  void operator()()
  {
    for (tentry e : tests_)
    {
      std::string name = std::get<0>(e);
      ttype test = std::get<1>(e);

      printf("%s...", name.c_str());
      try
      {
        test();
        printf("\tPASS\n");
      }
      catch (std::string err)
      {
        printf("\tFAILED\n");
        printf("\t%s\n", err.c_str());
      }
    }
  }
};

int main()
{

  TestRunner runner = TestRunner({
      {"test xxhash",
       []()
       {
         ASSERT(5308235351123835395 ==
                    hyperparameter::xxh64::hash(
                        "0123456789abcdefghijklmnopqrstuvwxyz", 36, 42),
                "xxhash not match");
       }},
      {"test param scope create",
       []()
       {
         auto hp = hyperparameter::create();
         ASSERT(hp != NULL, "error create hyperparameter");
       }},

      {"test param scope default value for undefined",
       []()
       {
         auto hp = hyperparameter::create_shared();
         ASSERT(1 == hp->get((uint64_t)1, 1), "default value is expected");
       }},
      {"test param scope put parameter",
       []()
       {
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
       []()
       {
         ASSERT(1 == GET_PARAM(a.b, 1), "get undefined param");
         {
           auto guard = WITH_PARAMS(a, 1,        //
                                    a.b, 2.0,    //
                                    a.b.c, true, //
                                    a.b.c.d, "str");
           ASSERT(1 == GET_PARAM(a, 0), "get int value");
           ASSERT(2.0 == GET_PARAM(a.b, 0.0), "get float value");
           ASSERT(true == GET_PARAM(a.b.c, false), "get bool value");
           ASSERT(std::string("str") == GET_PARAM(a.b.c.d, ""),
                  "get str value");
         }
       }},
      {"test WITH_PARAMS/GET_PARAM/GETPARAM",
       []()
       {
         ASSERT(1 == GET_PARAM(a.b, 1), "get undefined param");
         {
           auto guard = WITH_PARAMS(a, 1,        //
                                    a.b, 2.0,    //
                                    a.b.c, true, //
                                    a.b.c.d, "str");
           ASSERT(1 == GET_PARAM(a, 0), "get int value");
           ASSERT(1 == GETPARAM(a, 0), "get int value");
         }
       }},
      {"test nested WITH_PARAMS",
       []()
       {
         ASSERT(1 == GET_PARAM(a.b, 1), "get undefined param");
         {
           auto guard = WITH_PARAMS(a, 1,        //
                                    a.b, 2.0,    //
                                    a.b.c, true, //
                                    a.b.c.d, "str");
           ASSERT(1 == GET_PARAM(a, 0), "get int value");
           ASSERT(2.0 == GET_PARAM(a.b, 0.0), "get float value");
           ASSERT(true == GET_PARAM(a.b.c, false), "get bool value");
           ASSERT(std::string("str") == GET_PARAM(a.b.c.d, ""),
                  "get str value");
           {
             auto guard = WITH_PARAMS(a, 2, //
                                      a.b, 3.0);
             ASSERT(2 == GET_PARAM(a, 0), "get int value");
             ASSERT(3.0 == GET_PARAM(a.b, 0.0), "get float value");
           }
           ASSERT(1 == GET_PARAM(a, 0), "get int value");
           ASSERT(2.0 == GET_PARAM(a.b, 0.0), "get float value");
         }
       }},
      {"test bool parameters",
       []()
       {
         auto guard = WITH_PARAMS(a.true, true,   //
                                  a.false, false, //
                                  a.on, true,     //
                                  a.off, false,   //
                                  a.TRUE, true,   //
                                  a.False, false);
         ASSERT(true == GET_PARAM(a.true, false), "get bool value");
         ASSERT(false == GET_PARAM(a.false, true), "get bool value");
         ASSERT(true == GET_PARAM(a.on, false), "get bool value");
         ASSERT(false == GET_PARAM(a.off, true), "get bool value");
         ASSERT(true == GET_PARAM(a.TRUE, false), "get bool value");
         ASSERT(false == GET_PARAM(a.False, true), "get bool value");
       }},
  });
  runner();
  return 0;
}