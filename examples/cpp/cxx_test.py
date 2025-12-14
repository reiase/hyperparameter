from hyperparameter.librbackend import KVStorage
import hyperparameter as hp
import ctypes

a = ctypes.CDLL("./a.out")

with hp.scope() as ps:
    ps.test1.test2 = 1
    a.main()
    
    with hp.scope():
        hp.scope.test1.test2 = 2
        hp.scope.test1.bool1 = "true"
        hp.scope.test1.bool2 = "YES"
        hp.scope.test1.bool3 = "FALSE"
        hp.scope.test1.bool4 = "NO"
        a.main()