import Foundation

var standardError = FileHandle.standardError

extension FileHandle: TextOutputStream {
  public func write(_ string: String) {
    let data = Data(string.utf8)
    self.write(data)
  }
}

if let readValue = readLine() {
    if let value = Int(readValue) {
        let result = 2 * value
        print(result)
    } else {
        print("The input is not a number", standardError, to: &standardError)
    }
} else {
    print("Missing input\n", to: &standardError)
}
