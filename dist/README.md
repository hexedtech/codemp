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

* Memory management is entirely delegated to the JVM's garbage collector.
  * A more elegant solution than `Object.finalize()`, who is deprecated in newer Java versions, may be coming eventually.
* Exceptions coming from the native side have generally been made checked to imitate Rust's philosophy with `Result`.
  * `JNIException`s are however unchecked: there is nothing you can do to recover from them, as they usually represent a severe error in the glue code. If they arise, it's probably a bug.

### Using
`codemp` **will be available soon** as an artifact on [Maven Central](https://mvnrepository.com)

### Building
This is a [Gradle](https://gradle.org/) project: building requires having both Gradle and Cargo installed, as well as the JDK (any non-abandoned version).
Once you have all the requirements, building is as simple as running `gradle build`: the output is going to be a JAR under `build/libs`, which you can import into your classpath with your IDE of choice.
