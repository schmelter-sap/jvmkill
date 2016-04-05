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
#include <stdio.h>
#include <stdlib.h>
#include "heuristic.h"
#include "threshold.h"

bool testTriggersIfThresholdExceeded() {
	Heuristic *threshold = new Threshold("time=3,count=2");

	bool passed = !threshold->onOOM() &&
	              !threshold->onOOM() &&
	              threshold->onOOM();
    if (!passed) {
       fprintf(stdout, "testTriggersIfThresholdExceeded FAILED\n");
    }
	return passed;
}

bool testConstructionWithParameters() {
	Threshold *threshold = new Threshold("time=10,count=5");
	bool passed = ((threshold->getCount_Threshold() == 5) && (threshold->getTime_Threshold() == 10));
    if (!passed) {
       fprintf(stdout, "testConstructionWithParameters FAILED\n");
    }	return passed;
}

bool testConstructionWithNullPointer() {
	Threshold *threshold = new Threshold(0);
	bool passed = ((threshold->getCount_Threshold() == 0) && (threshold->getTime_Threshold() == 1));
    if (!passed) {
       fprintf(stdout, "testConstructionWithNullPointer FAILED\n");
    }	return passed;
}

bool testConstructionWithNoParameters() {
	Threshold *threshold = new Threshold("");
	bool passed = ((threshold->getCount_Threshold() == 0) && (threshold->getTime_Threshold() == 1));
    if (!passed) {
       fprintf(stdout, "testConstructionWithNoParameters FAILED\n");
    }	return passed;
}

int main() {
	bool result = (testTriggersIfThresholdExceeded() &&
		           testConstructionWithParameters() &&
              	   testConstructionWithNullPointer() &&
				   testConstructionWithNoParameters());
	if (result) {    	
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else { 
    	fprintf(stdout, "FAILURE\n");
    	exit(EXIT_FAILURE);
	}	
}

