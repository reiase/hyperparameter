header_path=$(python -c "import hyperparameter; print(hyperparameter.include)")
lib_path=$(python -c "import hyperparameter; print(hyperparameter.lib)")
g++ -O2 -std=c++14 -I ${header_path} cxx_test.cc ${lib_path} &&  python cxx_test.py
