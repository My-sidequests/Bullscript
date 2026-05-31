/// Print the help message.
pub fn run() {
    println!(
        "\
Available commands:

  build
      Enter build mode. You will be prompted for a filename, a function
      prototype, and up to 5 pipes. Type 'conclude' at any pipe prompt to
      finish early. The assembled function is pretty-printed for review
      before you choose whether to save it.

  run <file.bu>
      Execute a Bullang source file directly. The file must define a
      zero-argument 'main' bullet as its entry point. Any bullet containing
      a native escape block will be caught before execution and reported
      with a clear error — use 'bullarchy convert' for those instead.

  arrow <first> <second> <output_file>
      Apply one thing on another and write the result to a file.
      Two behaviors resolved automatically from the second argument:

        <second> is a program → pipe <first>'s stdout into <second>'s stdin.
        <second> is a file    → feed it as stdin to <first> directly.

      Always writes a result file. Useful for debugging, integration
      testing, or chaining any two programs together.

  test
      Interactively build a tester program for a compiled function.
      You will be prompted for a filename (language inferred from extension:
      rs, py, c, cpp, go), a function name, and a series of input/output
      pairs. Type 'conclude' to finish entering pairs. Then provide a tester
      name — Bullscript compiles and globally installs a Rust tester binary.
      Use it with arrow: arrow <tester> <your_program> result.txt

  update
      Reinstall Bullscript from the latest commit on the main branch.

  help
      Print this message.

  exit
      Quit Bullscript."
    );
}
