from unittest import TestCase
from threading import Thread

from hyperparameter import param_scope

class TestParamScopeThread(TestCase):
    def in_thread(self, key, val):
        ps = param_scope()
        print(getattr(ps, key)())
        self.assertEqual(getattr(ps, key)(), val)
    
    def test_new_thread(self):
        t = Thread(target=self.in_thread, args=("a.b", None))
        t.start()
        t.join
        
    # def test_new_thread_init(self):
    #     param_scope.A.B = 1
    #     param_scope.frozen()
    #     t = Thread(target=self.in_thread, args=("A.B", 1))
    #     t.start()
    #     t.join
        
if __name__ == "__main__":
    from unittest import main
    main()