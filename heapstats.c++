/*
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

#include <sys/types.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <iostream>
#include <map>

#include "heapstatshashtable.h"

std::multimap<char,size_t> javaObjects;

HeapStatsHashtable::HeapStatsHashtable() {
  
}

HeapStatsHashtable::~HeapStatsHashtable() {
}

void HeapStatsHashtable::recordObject(const char *className, size_t objectSize) {
    javaObjects.insert ( std::pair<char, size_t>(*className, objectSize) );
}

void HeapStatsHashtable::print(std::ostream& os) const {
    std::multimap<char, size_t>::const_iterator it;
    
    for (it=javaObjects.begin(); it!=javaObjects.end(); ++it) {
        os << (*it).first << " => " << (*it).second << '\n';
    }
}
