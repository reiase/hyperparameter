header_path=$(python -c "import hyperparameter; print(hyperparameter.include)")
lib_path="../../target/debug/libhyperparameter.a"
clang++ -O2 -std=c++17 -I ${header_path} test.cc ${lib_path} -o ctest &&  ./ctest
