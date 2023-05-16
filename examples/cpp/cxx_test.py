from hyperparameter.rbackend import KVStorage
from hyperparameter import param_scope
import ctypes

a = ctypes.CDLL("./a.out")

with param_scope() as ps:
    ps.test1.test2 = 1
    a.main()
    
    with param_scope():
        param_scope.test1.test2 = 2
        a.main()