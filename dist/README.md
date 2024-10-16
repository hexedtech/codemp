# Compiling and Distributing FFI-compatible binaries
`codemp` aims to target as many platforms as possible, while remaining maintainable and performant.

To guarantee this, it can compile to a bare rust lib but also 4 different FFI-compatible shared objects: JavaScript, Python, Lua, Java.

> We also plan to offer bare C bindings for every other language which can do C interop, but it's not our top priority right now.

To compile the bare FFI-compatible shared object, just `cargo build --release --features=<lang>`, replacing `<lang>` with either `js`, `py`, `java`, `lua` or `luajit`.
In most languages, just importing the resulting shared object will work, however refer to each language's section below for more in-depth information.

## JavaScript
To build a npm package, `napi-cli` must first be installed: `npm install napi-cli`.

You can then `npx napi build` in the project root to compile the native extension and create the type annotations (`index.d.ts`).
A package.json is provided for publishing, but will require some tweaking.

## Python
To distribute the native extension we can leverage python wheels. It will be necessary to build the relevant wheels with [`maturin`](https://github.com/PyO3/maturin).
After installing with `pip install maturin`, run `maturin build` to obtain an `import`able package and installable wheels.

## Lua
Built Lua bindings are valid lua modules and require no extra steps to be used.

## Java
`codemp`'s Java bindings are implemented using the [JNI](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/).

On the Rust side, all Java-related code is gated behind the `java` feature, and is implemented using [`jni`](https://github.com/jni-rs/jni-rs) crate.

Unlike other supported languages, Java is statically typed and requires knowing all foreign function types at compile time.
This means that, to use `codemp` through the JNI, all functions who need to be called must also be declared on the Java side, marked as `native`.

Thus, we also provide pre-made Java glue code, wrapping all native calls and defining classes to hold `codemp` types.

The Java bindings have no known major quirk. However, here are a list of facts that are useful to know when developing with these:

* Memory management is entirely delegated to the JVM's garbage collector using the `Cleaner` API.
  * Because of this, we require Java 11 as minimum version: `Cleaner` was added in version 9. This should not be a problem, as IDEs tend to run on recent versions, but if there is actual demand for it we may add a Java 8-friendly version using `Object.finalize()` (which is deprecated in modern JDKs).
* Exceptions coming from the native side have generally been made checked to imitate Rust's philosophy with `Result`.
  * `JNIException`s are however unchecked: there is nothing you can do to recover from them, as they usually represent a severe error in the glue code. If they arise, it's probably a bug.

### Using
`codemp` is available on [Maven Central](https://central.sonatype.com/artifact/mp.code/codemp), with each officially supported OS as an archive classifier.

### Building
> [!NOTE]
> The following instructions assume `dist/java` as current working directory.

This is a [Gradle](https://gradle.org/) project, so you must have install `gradle` (as well as JDK 11 or higher) in order to build it.
- You can build a JAR without bundling the native library with `gradle build`.
- Otherwise, you can compile the project for your current OS and create a JAR that bundles the resulting binary with `gradle nativeBuild`; do note that this second way of building also requires Cargo and the relevant Rust toolchain.

In both cases, the output is going to be a JAR under `build/libs`, which you can import into your classpath with your IDE of choice.
