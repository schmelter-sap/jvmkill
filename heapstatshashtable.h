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
 
#ifndef heapstatshashtable_h
#define heapstatshashtable_h

#include "heapstats.h"
#include <unordered_map>
#include <algorithm>
#include <vector>
#include <map>

struct ObjectCount {
    size_t objectSize;
    size_t objectCount;
};

class HeapStatsHashtable: public HeapStats {
public:
    HeapStatsHashtable(int maxEntries);
    
    virtual ~HeapStatsHashtable();

    void recordObject(const char *className, size_t objectSize);
  
    void print(std::ostream& os) const;

private:
    int heapHistogramMaxEntries;
    std::unordered_map<std::string, ObjectCount> javaObjects;
};

class HeapStatsHashtableFactory: public HeapStatsFactory {
public:
    HeapStatsHashtableFactory(int maxEntries) {
        heapHistogramMaxEntries = maxEntries;
    }

    virtual ~HeapStatsHashtableFactory() {}

    HeapStats* create() {
        return new HeapStatsHashtable(heapHistogramMaxEntries);
    }

private:
    int heapHistogramMaxEntries;
};

#endif // heapstatshashtable_h
