from bindings import exports

state: int = 0

class PackNameApi(exports.PackNameApi):
    def add(value: int):
      global state
      print("add " + str(value))
      state = state + value 

    def get() -> int:
       global state
       print("get")
       return state
