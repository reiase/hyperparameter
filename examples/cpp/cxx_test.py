from hyperparameter.librbackend import KVStorage
from hyperparameter import param_scope
import ctypes

a = ctypes.CDLL("./a.out")

with param_scope() as ps:
    ps.test1.test2 = 1
    a.main()
    
    with param_scope():
        param_scope.test1.test2 = 2
        param_scope.test1.bool1 = "true"
        param_scope.test1.bool2 = "YES"
        param_scope.test1.bool3 = "FALSE"
        param_scope.test1.bool4 = "NO"
        a.main()