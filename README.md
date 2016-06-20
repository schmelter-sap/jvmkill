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

# Building

    make JAVA_HOME=/path/to/jdk

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

## printHeapHistogram

Determines whether or not a histogram of heap usage is printed before the agent kills the JVM.
To enable histogram printing, set the parameter to 1. Defaults to 0 (disabled) if not provided. 

## time
Configures the time limit (in seconds) in which resourceExhausted events are kept in 
the counter. Defaults to 1 if not provided.
