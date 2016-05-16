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

#include <stdexcept>
#include <stdlib.h>
#include <string.h>

#include "heaphistogramaction.h"

HeapHistogramAction* heapHistogramAction;

struct jvmtiInterface_1_ mockJvmtiInterface_1_;
jvmtiEnv mockJvmtiEnvStruct;
jvmtiEnv *mockJvmtiEnv;

static int MockGetCapabilitiesCount;
static jvmtiError MockGetCapabilitiesReturnValue;
jvmtiError (JNICALL MockGetCapabilities) (jvmtiEnv* env, 
	jvmtiCapabilities* capabilities) {
	MockGetCapabilitiesCount++;
	capabilities->can_generate_garbage_collection_events = 0;
	return MockGetCapabilitiesReturnValue;
}

static int MockAddCapabilitiesCount;
static jvmtiError MockAddCapabilitiesReturnValue;
static jvmtiCapabilities MockAddedCapabilities;
jvmtiError (JNICALL MockAddCapabilities) (jvmtiEnv* env, 
	const jvmtiCapabilities* capabilities) {
	MockAddCapabilitiesCount++;
	memcpy(&MockAddedCapabilities, capabilities, sizeof(jvmtiCapabilities));
	return MockAddCapabilitiesReturnValue;
}

void setup() {
	mockJvmtiEnvStruct.functions = &mockJvmtiInterface_1_;

	MockGetCapabilitiesCount = 0;
	MockGetCapabilitiesReturnValue = JVMTI_ERROR_NONE;
	((struct jvmtiInterface_1_ *)mockJvmtiEnvStruct.functions)->GetCapabilities = &MockGetCapabilities;
	
	MockAddCapabilitiesCount = 0;
	MockAddCapabilitiesReturnValue = JVMTI_ERROR_NONE;
	((struct jvmtiInterface_1_ *)mockJvmtiEnvStruct.functions)->AddCapabilities = &MockAddCapabilities;

	mockJvmtiEnv = &mockJvmtiEnvStruct;
}

void teardown() {
	// placeholder in case setup acquires resources
}

bool testConstructionOk() {
	setup();

	heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv);
	bool passed = (MockGetCapabilitiesCount == 1) &&
					(MockAddCapabilitiesCount == 1) &&
					(MockAddedCapabilities.can_tag_objects == 1);
	if (!passed) {
		fprintf(stdout, "testConstruction FAILED\n");
	}

	delete(heapHistogramAction);
	teardown();
	return passed;
}

bool testConstructionGetCapabilitiesFailure() {
	setup();
	MockGetCapabilitiesReturnValue = JVMTI_ERROR_ACCESS_DENIED;

    bool passed = false;
    try {
		heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv);
    } catch (std::runtime_error *re) {
    	passed = (MockGetCapabilitiesCount == 1) &&
    				(MockAddCapabilitiesCount == 0);
	}
	if (!passed) {
		fprintf(stdout, "testConstructionGetCapabilitiesFailure FAILED\n");
	}

	teardown();
	return passed;
}

bool testConstructionAddCapabilitiesFailure() {
	setup();
	MockAddCapabilitiesReturnValue = JVMTI_ERROR_ACCESS_DENIED;

    bool passed = false;
    try {
		heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv);
    } catch (std::runtime_error *re) {
    	passed = (MockGetCapabilitiesCount == 1) &&
    				(MockAddCapabilitiesCount == 1);
	}
	if (!passed) {
		fprintf(stdout, "testConstructionAddCapabilitiesFailure FAILED\n");
	}

	teardown();
	return passed;
}

int main() {
	bool result = (testConstructionOk() &&
						testConstructionGetCapabilitiesFailure() &&
						testConstructionAddCapabilitiesFailure());
	if (result) {
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else {
    	fprintf(stdout, "FAILURE\n");
    	exit(EXIT_FAILURE);
	}
}