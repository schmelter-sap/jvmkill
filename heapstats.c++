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
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <iostream>
#include <unordered_map>

#include "heapstatshashtable.h"

std::unordered_map<std::string, size_t> javaObjects;
int longestClassName = 10;

HeapStatsHashtable::HeapStatsHashtable() {
}

HeapStatsHashtable::~HeapStatsHashtable() {
    javaObjects.clear();
    longestClassName = 10;
}

void HeapStatsHashtable::recordObject(const char *className, size_t objectSize) {
    const std::string classNameString (className);
    if (javaObjects.count(classNameString) > 0) {
        objectSize = javaObjects.at(classNameString) + objectSize;
        javaObjects.erase(classNameString);
    }
    if(classNameString.length() > longestClassName) {
        longestClassName = classNameString.length();
    }
    javaObjects.insert ( std::pair<std::string, size_t>(classNameString, objectSize) );
}

void HeapStatsHashtable::print(std::ostream& os) const {
    std::unordered_map<std::string, size_t>::const_iterator it;
    
    std::string heading = "Class Name";
    heading.resize(longestClassName, ' ');
    
    os << "| " << heading << " | Size of Objects |\n";
    
    for (it=javaObjects.begin(); it!=javaObjects.end(); ++it) {
        std::string name = (*it).first;
        name.resize(longestClassName, ' ');
        
        std::string totalSize = std::to_string((*it).second);
        totalSize.resize(15, ' ');
     
        os << "| " << name << " | " << totalSize << " |\n";
    }
}
