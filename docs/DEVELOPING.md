# Developers' Guide

jvmkill is a [JVMTI][] agent written in Rust and built using header files provided by the JDK.

[JVMTI]: http://docs.oracle.com/javase/8/docs/technotes/guides/jvmti/

## Dependencies

jvmkill has the following dependencies:

* Rust for compilation and linking
* JDK for JVMTI and JNI header files

Development was undertaken on MacOS 12.13 (Monterey) using Rust 1.60.0 and Azul Zulu JDK 1.8.0_332.

The jvmkill ci pipeline builds the agent for various platforms (at the time of writing: Ubuntu Trusty and Ubuntu Bionic). The pipeline definition is stored in the [Java Experience Concourse git repo][] (a private repository).

[Java Experience Concourse git repo]: https://github.com/pivotal-cf/java-experience-concourse

## Code Structure

The code is rooted in the `Agent_OnLoad` function in [lib.rs][]. This function is called when the JVM loads the agent.

The agent registers a `resource_exhausted` function with the JVM which is called whenever the JVM encounters a resource exhausted event.

The `resource_exhausted` function enters a mutex, calls the `AgentController` struct which delegates to various `Action` trait objects to act on the resource exhaustion, and then exits the mutex before returning. One of the actions which may be driven is to kill the JVM, in which case the JVM process is killed, including the current thread, and there will be no return to the `AgentController` class.

The following sequence diagram shows some typical interactions between the JVM and the jvmkill agent:

![Sequence diagram](jvmkill.png)

[lib.rs]: src/lib.rs

## Testing

jvmkill has unit and other tests. See the [Testing][] section of the README for how to run the tests.

[Building]: ../README.md#Testing

## Contributing

Please refer to the [Contributors' Guide][].

[Contributors' Guide]: CONTRIBUTING.md

## Community

Others involved in jvmkill development use the `#java-buildpack` channel of the  [Cloud Foundry slack organisation][] for discussion.

[Cloud Foundry slack organisation]: https://cloudfoundry.slack.com
