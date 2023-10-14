#include <iostream>
#include "hyperparameter.h"

int main()
{
    auto hp = hyperparameter::create();

    std::cout << "\n:: test xxhash" << std::endl
              << "expected: 5308235351123835395" << std::endl
              << "returned: "
              << hyperparameter::xxh64::hash("0123456789abcdefghijklmnopqrstuvwxyz", 36, 42) << std::endl;

    std::cout << "\n:: test undefined" << std::endl
              << "expected: 1" << std::endl
              << "returned: "
              << hp->get((uint64_t)1, 1) << std::endl;

    hp->put("a", 2);
    hp->put("x.y.z", true);
    std::cout << "\n:: test put parameter" << std::endl
              << "expected: 2" << std::endl
              << "returned: "
              << hp->get("a", 1.0) << std::endl
              << "expected: 1" << std::endl
              << "returned: "
              << hp->get("x.y.z", false) << std::endl;

    hp->put("a", "str:2");
    hp->put("x.y.z", "str:true");
    std::string a = hp->get("a", "1");
    printf("a=%s\n", a.c_str());
    std::cout << "\n:: test put str parameter" << std::endl
              << "expected: str:2" << std::endl
              << "returned: " << a << std::endl
              << "expected: str:true" << std::endl
              << "returned: "
              << hp->get("x.y.z", "false") << std::endl;
    delete hp;
    // ======= opt api test =======

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