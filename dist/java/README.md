# Java bindings
`codemp`'s Java bindings are implemented using the [JNI](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/).

On the Rust side, all Java-related code is gated behind the `java` feature, and is implemented using[`jni-rs`](https://github.com/jni-rs/jni-rs).
Unlike other languages, Java requires glue code on both sides: as a result, a Java component is necessary.

## Building
This is a [Gradle](https://gradle.org/) project: building requires having both Gradle and Cargo installed, as well as the JDK (any non-abandoned version).
Once you have all the requirements, building is as simple as running `gradle build`: the output is going to be a JAR under `build/libs`, which you can import into your classpath with your IDE of choice.

## Development
The Java bindings have no known major quirk. However, here are a list of facts that are useful to know when developing with these:

* Memory management is entirely delegated to the JVM's garbage collector.
  * A more elegant solution than `Object.finalize()`, who is deprecated in newer Java versions, may be coming eventually.
* Exceptions coming from the native side have generally been made checked to imitate Rust's philosophy with `Result`.
  * `JNIException`s are however unchecked: there is nothing you can do to recover from them, as they usually represent a severe error in the glue code. If they arise, it's probably a bug.
