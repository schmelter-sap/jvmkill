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

#include <string>
#include <sstream>
#include <iostream>

#include "heapstatshashtable.h"

HeapStatsHashtable *heapStats;

const char* test_class_name = "java.lang.String";
const std::string test_result ("j => 24\n");

void setup() {
    heapStats = new HeapStatsHashtable();
}

void teardown() {
    if (heapStats != NULL) {
        delete heapStats;    
    }
}

//bool testRecordObject() {
//    bool result = false;
//    setup();
//    HeapStatsHashtable tHeapStats = *heapStats;
//    tHeapStats.recordObject(test_class_name, 24);
//    result = true;
//    teardown();
//    return result;
//}

bool testRecordAndPrint() {
    bool result = false;
    setup();
    HeapStatsHashtable tHeapStats = *heapStats;
    std::stringstream ss;
    
    tHeapStats.recordObject(test_class_name, 24); 
    tHeapStats.print(ss);
    
//    std::cout << "***********\n";
//    std::cout << ss.str().c_str();
//    std::cout << "***********\n";
//    std::cout << test_result;
//    std::cout << "***********\n";
    
    if(test_result.compare(ss.str()) == 0) {
        result = true;
    }
    teardown();
    return result;
}

int main() {
    bool result = testRecordAndPrint();
    if (result) {    	
        fprintf(stdout, "SUCCESS\n");
        exit(EXIT_SUCCESS);
    }
    else { 
        fprintf(stdout, "FAILURE\n");
        exit(EXIT_FAILURE);
    }	
}
