ifndef JAVA_HOME
    $(error JAVA_HOME not set)
endif

ifeq ($(shell uname -s),Darwin)
  INCLUDE= -I"$(JAVA_HOME)/include" -I"$(JAVA_HOME)/include/darwin"
  LINK= -Wl,-install_name,libJvmKill
else
  INCLUDE= -I"$(JAVA_HOME)/include" -I"$(JAVA_HOME)/include/linux"
  LINK= -Wl,-soname=libJvmKill,--no-as-needed -static-libgcc
endif

CFLAGS=-Wall -Werror -fPIC -shared -fno-strict-aliasing $(LINK) -fno-omit-frame-pointer $(INCLUDE)
CPPFLAGS=-std=c++11 $(CFLAGS)
TARGET=libjvmkill.so

.PHONY: all build clean alltests ctests threadtests threadtestbasic threadtest0 threadtest-10-2 memtests memtest0 memtest-10-2

all: build alltests

build:	
	@echo "=============================================="
	g++ $(CPPFLAGS) -o $(TARGET) jvmkill.c++ threshold.c++ killaction.c++
	chmod 644 $(TARGET)

clean:
	@echo "=============================================="
	rm -f $(TARGET)
	rm -f *.class
	rm -f *.hprof
	rm -rf *.dSYM
	rm -f *tests

alltests: ctests threadtests memtests

ctests: build thresholdctests killactionctests
	@echo "=============================================="
	gcc $(CFLAGS) -g $(INCLUDE) -ldl -o tests tests.c
	./tests

thresholdctests: build
	g++ $(CPPFLAGS) -g $(INCLUDE) -ldl -o thresholdtests thresholdtests.c++ threshold.c++
	./thresholdtests

killactionctests: build
	g++ $(CPPFLAGS) -g $(INCLUDE) -ldl -o killactiontests killactiontests.c++ killaction.c++
	./killactiontests

threadtests: threadtestbasic threadtest0 threadtest-10-2 threadtestpspawn-10-2

threadtest0: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreads.java
	!($(JAVA_HOME)/bin/java -Xmx1m \
	    -agentpath:$(PWD)/$(TARGET) \
	    -cp $(PWD) JvmKillTestThreads)

threadtest-10-2: build
	@echo "=============================================="
	!($(JAVA_HOME)/bin/java -Xmx1m \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2 \
	    -cp $(PWD) JvmKillTestThreads)

memtests: memtest0 memtest-10-2

memtest0: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest.java
	!($(JAVA_HOME)/bin/java -Xmx5m \
	    -agentpath:$(PWD)/$(TARGET) \
	    -cp $(PWD) JvmKillTest)


memtest-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest.java
	($(JAVA_HOME)/bin/java -Xmx5m \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2 \
	    -cp $(PWD) JvmKillTest)

threadtestbasic: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTest2.java
	ulimit -u 
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET) \
	    -cp $(PWD) JvmKillTest2)

threadtestpspawn-10-2: build
	@echo "=============================================="
	$(JAVA_HOME)/bin/javac JvmKillTestThreadsParallel.java
	!($(JAVA_HOME)/bin/java \
	    -agentpath:$(PWD)/$(TARGET)=time=10,count=2 \
	    -cp $(PWD) JvmKillTestThreadsParallel)
