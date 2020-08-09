With Compiler checks:
1. User creates a DAG in some serialized format
2. Pre-build script reads the serialized data and writes a rust program
3. Invoke rust compiler on written rust program
4. Call the compiled binary with the serialized DAG


Without compiler checks:
1. User creates a DAG in some serialized format
2. Call the compiled binary with the serialized DAG



- Do we get enough help from the type system to justify asking users to install the rust compiler on their systems?
- Rust compilation is slow

