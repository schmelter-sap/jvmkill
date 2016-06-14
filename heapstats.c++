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
#include <algorithm>
#include <vector>
#include <map>

#include "heapstatshashtable.h"

unsigned int longestClassName = 10;

struct ObjectCount {
    size_t objectSize;
    size_t objectCount;
};

std::unordered_map<std::string, ObjectCount> javaObjects;

HeapStatsHashtable::HeapStatsHashtable() {
}

HeapStatsHashtable::~HeapStatsHashtable() {
    javaObjects.clear();
    longestClassName = 10;
}

void HeapStatsHashtable::recordObject(const char *className, size_t objectSize) {
    const std::string classNameString (className);
    size_t objectCount = 1;
    if (javaObjects.count(classNameString) > 0) {
        objectSize = javaObjects.at(classNameString).objectSize + objectSize;
        objectCount = javaObjects.at(classNameString).objectCount + objectCount;
        javaObjects.erase(classNameString);
    } else if(classNameString.length() > longestClassName) {
        longestClassName = classNameString.length();
    }
    javaObjects.insert (std::pair<std::string, ObjectCount>(classNameString, {objectSize, objectCount}) );
}

bool sorter (std::pair<std::string, ObjectCount> i, std::pair<std::string, ObjectCount> j) { 
    return (i.second.objectSize > j.second.objectSize); 
}

void HeapStatsHashtable::print(std::ostream& os) const {
    std::vector<std::pair<std::string, ObjectCount> > tmpObjects(javaObjects.begin(), javaObjects.end());    
    std::sort(tmpObjects.begin(), tmpObjects.end(), sorter);

    std::string heading = "Class Name";
    heading.resize(longestClassName, ' ');
    os << "| Instance Count | Total Bytes | " << heading << " |\n";
    
    for (std::vector<std::pair<std::string, ObjectCount> >::iterator it=tmpObjects.begin(); it!=tmpObjects.end(); it++) {        
        (*it).first.resize(longestClassName, ' ');
        std::string totalSize = std::to_string((*it).second.objectSize);
        totalSize.resize(11, ' ');    
        std::string totalCount = std::to_string((*it).second.objectCount);
        totalCount.resize(14, ' ');
        
        os << "| " << totalCount << " | " << totalSize << " | " << (*it).first << " |\n";
    }
    
}
