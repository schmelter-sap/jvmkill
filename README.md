[Concourse Pipeline](https://java-experience.ci.springapps.io/pipelines/jvmkill)

# Overview

**jvmkill** is a simple [JVMTI][] agent that forcibly terminates the JVM
when it is unable to allocate memory or create a thread. This is important
for reliability purposes: an `OutOfMemoryError` will often leave the JVM
in an inconsistent state. Terminating the JVM will allow it to be restarted
by an external process manager.

[JVMTI]: http://docs.oracle.com/javase/8/docs/technotes/guides/jvmti/

It is often useful to automatically dump the Java heap using the
`-XX:+HeapDumpOnOutOfMemoryError` JVM argument. This agent will be
notified and terminate the JVM after the heap dump completes.

A common alternative to this agent is to use the
`-XX:OnOutOfMemoryError` JVM argument to execute a `kill -9` command.
Unfortunately, the JVM uses the `fork()` system call to execute the kill
command and that system call can fail for large JVMs due to memory
overcommit limits in the operating system.  This is the problem that
motivated the development of this agent.

# Pre-requisites

Install [Rust][] v1.19.0 or later.

Ensure that you have libclang v3.9 or later installed. To install this on Ubuntu Trusty see [ask ubuntu][]
and set `$LIBCLANG_PATH` to the directory containing (v3.9 or later of) `libclang.so` and `libclang.so.1`.

[Rust]: https://www.rust-lang.org/en-US/install.html
[ask ubuntu]: https://askubuntu.com/questions/787383/how-to-install-llvm-3-9
    
# Building

To build the agent, install the above pre-requisites, then issue:

    cargo build --release -p jvmkill
    
# Testing

To run the tests, install the above pre-requisites, then issue:

    cargo test --all
    
# Usage

Run Java with the agent added as a JVM argument:

    -agentpath:/path/to/libjvmkill.so=<parameters>

Alternatively, if modifying the Java command line is not possible, the
above may be added to the `JAVA_TOOL_OPTIONS` environment variable.

# Agent parameters

The agent configurations can be passed using the standard agent mechanism.
The parameters should be passed as a comma separated string. Eg.: count=2,time=10
The agent accepts the following parameters:

## count

Configures the limit of resourceExhausted events that can be fired in the configured
time interval. Defaults to 0 if not provided (JVM is killed with a single fired event).

## time

Configures the time limit (in seconds) in which resourceExhausted events are kept in 
the counter. Defaults to 1 if not provided.

## heapDumpPath

Configures a file to which a heap dump is written before the agent kills the JVM.

To enable heap dump generation, set the parameter to the path of a file in a writable directory with sufficient free space.
The path may be absolute or relative to the working directory where the JVM was started.
If the parameter is not specified, no heap dump is generated.

The path is treated as a [strftime](https://linux.die.net/man/3/strftime) format specification,
although the precise set of format codes supported depends on the platform.
For example, the string "%a-%d-%b-%Y-%T-%z" approximates the date format of RFC 2822 while avoiding embedded spaces
(which are awkward in agent parameters).

A heap dump (of live objects only) is generated if a path is specified _and_ the 
[HotSpot Diagnostic MXBean](https://docs.oracle.com/javase/8/docs/jre/api/management/extension/com/sun/management/HotSpotDiagnosticMXBean.html)
is available.
If the parent directories of the path do not exist, they are created.

If the file exists before the heap dump is produced, it is overwritten. Including the date and time
in the file path, using strftime format codes, may reduce the risk that the file already exists.

## printHeapHistogram

Determines whether or not a histogram of heap usage is printed before the agent kills the JVM.
To enable histogram printing, set the parameter to 1. Defaults to 0 (disabled) if not provided.

Each entry in the histogram describes the number of instances of a particular Java type, the
total number of bytes in the heap consumed by those instances, and the name of the type.

The histogram is sorted in order of decreasing total number of bytes.

The histogram may be truncated. To set the number of entries that appear, use the `heapHistogramMaxEntries` parameter.
 
## heapHistogramMaxEntries

When histogram printing is enabled, limits the number of entries in the histogram to the value
of the parameter. Defaults to 100 if not provided. Set the parameter to 0 to print the entire histogram.

## printMemoryUsage

Determines whether or not memory usage is printed before the agent kills the JVM.
To disable memory usage printing, set the parameter to 0. Defaults to 1 (enabled) if not provided.

If the agent has been driven because the JVM is unable to create a thread, memory usage is not printed
as attempting to obtain memory usage statistics can cause the agent to fail in which case the JVM is not killed.

When testing thread exhaustion with a small heap on Linux, it was found that
the agent can be driven for heap exhaustion and yet
obtaining memory usage stats can still cause the agent to fail in which case
the JVM is not killed. If this is encountered with a real application, printing memory
usage can be disabled.

## License

The jvmkill agent is Open Source software released under the
[Apache 2.0 license](http://www.apache.org/licenses/LICENSE-2.0.html).

## Development

Please refer to the [Developers' Guide][].

[Developers' Guide]: docs/DEVELOPING.md
