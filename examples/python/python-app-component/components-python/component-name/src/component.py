state: int = 0

from binding.pack_name import exports
from lib import example_common_function

class ComponentNameApi(exports.ComponentNameApi):
    def add(self, value: int):
        global state
        print("add " + str(value))
        print(example_common_function())
        state = state + value

    def get(self) -> int:
        global state
        print("get")
        return state
