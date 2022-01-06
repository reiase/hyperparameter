Overview
========

hyperparameter 是一个参数管理工具库，用于解决如下问题：
1. 应用级别/库级别参数配置与管理；
2. 机器学习模型超参配置和调优；
3. AutoML参数寻优；

Quick Start
===========

``` python
def foo():
    with param_scope() as hp:
        return hp.a
    
with param_scope(a=1, b=2) as hp:
    hp() # 1
```