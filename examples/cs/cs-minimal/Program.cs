
// Currently .NET WASI does not support reading from stdin or using the command line aruments,
// so the only way for us to pass some parameters to the program is through environment variables.
String input = System.Environment.GetEnvironmentVariable("INPUT");

try {
    int result = 2 * Convert.ToInt32(input);
    Console.WriteLine(result);
} catch {
    Console.Error.WriteLine($"Input ${input} was not a number");
}
