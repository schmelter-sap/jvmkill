ifndef JAVA_HOME
    $(error JAVA_HOME not set)
endif

ifeq ($(shell uname -s),Darwin)
  TARGET=target/release/libjvmkill.dylib
else
  TARGET=target/release/libjvmkill.so
endif

.PHONY: all build clean alltests threadtests threadtestbasic threadtest0 threadtest-10-2 threadtestpspawn-10-2 memtests memtest0 memtest-10-2

all: alltests build

build:
	cargo build --release

clean:
	cargo clean
	rm -f *.class

alltests: cargotests threadtests memtests

cargotests:
	cargo test --all

threadtests: threadtestbasic threadtest0 threadtest-10-2 threadtestpspawn-10-2

threadtestbasic: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest2.java
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET) \
	    -cp $(PWD) JvmKillTest2)

threadtest0: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreads.java
	!($(JAVA_HOME)/bin/java -Xmx1m \
	    -agentpath:$(PWD)/$(TARGET)=printMemoryUsage=0 \
	    -cp $(PWD) JvmKillTestThreads)

threadtest-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreads.java
	!($(JAVA_HOME)/bin/java -Xmx1m \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2,printHeapHistogram=1,heapHistogramMaxEntries=10,printMemoryUsage=0 \
	    -cp $(PWD) JvmKillTestThreads)

threadtestpspawn-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreadsParallel.java
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2 \
	    -cp $(PWD) JvmKillTestThreadsParallel)

memtests: memtest0 memtest-10-2

memtest0: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest.java
	!($(JAVA_HOME)/bin/java -Xmx5m \
	    -agentpath:$(PWD)/$(TARGET)=printHeapHistogram=1,heapHistogramMaxEntries=20 \
	    -cp $(PWD) JvmKillTest)

memtest-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest.java
	!($(JAVA_HOME)/bin/java -Xmx5m \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2,heapDumpPath=/tmp/jbp/dump-%a-%d-%b-%Y-%T-%z.hprof,printHeapHistogram=1,heapHistogramMaxEntries=10 \
	    -cp $(PWD) JvmKillTest)
