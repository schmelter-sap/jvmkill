ifndef JAVA_HOME
    $(error JAVA_HOME not set)
endif

ifeq ($(shell uname -s),Darwin)
  INCLUDE= -I"$(JAVA_HOME)/include" -I"$(JAVA_HOME)/include/darwin"
  LINK= -Wl,-install_name,libJvmKill
else
  INCLUDE= -I"$(JAVA_HOME)/include" -I"$(JAVA_HOME)/include/linux"
  LINK= -Wl,--no-as-needed,-soname=libJvmKill -static-libgcc
endif

CPPFLAGS=-Wall -Werror -fPIC -std=c++11 -fno-strict-aliasing $(LINK) -fno-omit-frame-pointer $(INCLUDE)
CPPFLAGS_SO=$(CPPFLAGS) -shared
CPPFLAGS_TEST=$(CPPFLAGS) -ldl
TARGET=libjvmkill.so

.PHONY: all build clean alltests ctests threadtests threadtestbasic threadtest0 threadtest-10-2 memtests memtest0 memtest-10-2

all: build alltests

build:
	@echo "=============================================="
	g++ $(CPPFLAGS_SO) -o $(TARGET) jvmkill.c++ agentcontroller.c++ parametersparser.c++ threshold.c++ killaction.c++ heaphistogramaction.c++ poolstatsaction.c++ heapstats.c++
	chmod 644 $(TARGET)

clean:
	@echo "=============================================="
	rm -f $(TARGET)
	rm -f *.class
	rm -f *.hprof
	rm -rf *.dSYM
	rm -f *tests

alltests: ctests threadtests memtests

ctests: thresholdctests killactionctests agentcontrollerctests heaphistogramactionctests heapstatsctests parameterparserctests

thresholdctests:
	g++ $(CPPFLAGS_TEST) -o thresholdtests thresholdtests.c++ threshold.c++
	./thresholdtests

killactionctests:
	g++ $(CPPFLAGS_TEST) -o killactiontests killactiontests.c++ killaction.c++
	./killactiontests

agentcontrollerctests:
	g++ $(CPPFLAGS_TEST) -Wno-unused-private-field -o agentcontrollertests agentcontrollertests.c++ agentcontroller.c++
	./agentcontrollertests

heaphistogramactionctests:
	g++ $(CPPFLAGS_TEST) -o heaphistogramactiontests heaphistogramactiontests.c++ heaphistogramaction.c++
	./heaphistogramactiontests

heapstatsctests:
	g++ $(CPPFLAGS_TEST) -o heapstatstests heapstatstests.c++ heapstats.c++
	./heapstatstests

parameterparserctests:
	g++ $(CPPFLAGS_TEST) -o parameterparsertests parametersparsertests.c++ parametersparser.c++
	./parameterparsertests

threadtests: threadtestbasic threadtest0 threadtest-10-2 threadtestpspawn-10-2

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
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2,printHeapHistogram=1,heapHistogramMaxEntries=10 \
	    -cp $(PWD) JvmKillTest)

threadtestbasic: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest2.java
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET) \
	    -cp $(PWD) JvmKillTest2)

threadtestpspawn-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreadsParallel.java
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2 \
	    -cp $(PWD) JvmKillTestThreadsParallel)
