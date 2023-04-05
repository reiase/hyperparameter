#include <iostream>
#include "hyperparameter.h"

int main()
{
    auto hp = hyperparameter::create_shared();

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

    std::cout << "in main" << std::endl;
    return 0;
}