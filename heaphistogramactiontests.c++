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

#include <stdlib.h>

#include "heaphistogramaction.h"

HeapHistogramAction* heapHistogramAction;

void setup() {
	heapHistogramAction = new HeapHistogramAction(NULL);
}

void teardown() {
	delete(heapHistogramAction);
}

bool testConstruction() {
	setup();
	bool passed = true;
	if (!passed) {
		fprintf(stdout, "testConstruction FAILED\n");
	}
	teardown();
	return passed;
}

int main() {
	bool result = (testConstruction() &&
						testConstruction());
	if (result) {
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else {
    	fprintf(stdout, "FAILURE\n");
    	exit(EXIT_FAILURE);
	}
}