from __future__ import print_function
try:
    # python2
    import __builtin__
except ImportError:
    # python3
    import builtins as __builtin__
    
def print(*args, **kwargs):
    return __builtin__.print(*args, **kwargs, flush=True)
input_args = input()
print(input_args)