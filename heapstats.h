/*
 * Copyright (c) 2015 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
 
#ifndef heapstats_h
#define heapstats_h

#include <sys/types.h>
#include <signal.h>
#include <stdexcept>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <iostream>

class HeapStats {
public:
    // Destructor
    virtual ~HeapStats() {};
  
    /* Record the statistics for a single object: the class name (in internal JVM format) 
    and the object size in bytes.
     */
    virtual void recordObject(const char *className, size_t objectSize) = 0;
  
    // Print a histogram of the heap statistics to the given output stream.
    virtual void print(std::ostream& os) const = 0;
};

class HeapStatsFactory {
public:
    // Destructor
    virtual ~HeapStatsFactory() {};

    virtual HeapStats* create() = 0;
};

#endif // heapstats_h

