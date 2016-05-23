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

#include "heapstatshashtable.h"

HeapStats *heapStats;

void setup() {
  heapStats = new HeapStatsHashtable();
}

void teardown() {
  if (heapStats != NULL) {
    delete heapStats;    
  }
}

bool testRecordObject() {
	setup();
	teardown();
	return false;
}

bool testPrint() {
	setup();
	teardown();
	return false;
}

int main() {
	bool result = testRecordObject() && testPrint();
	if (result) {    	
    fprintf(stdout, "SUCCESS\n");
    exit(EXIT_SUCCESS);
	}
	else { 
    fprintf(stdout, "FAILURE\n");
    exit(EXIT_FAILURE);
	}	
}
