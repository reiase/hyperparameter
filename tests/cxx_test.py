from hyperparameter.rbackend import KVStorage
import ctypes

a = ctypes.CDLL("./a.out")
a.main()