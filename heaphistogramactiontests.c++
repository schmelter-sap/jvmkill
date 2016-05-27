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
#include "heapstats.h"

HeapHistogramAction* heapHistogramAction;

JNIEnv* mockJNIEnv;

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

static int MockGetLoadedClassesCount;
static jint MockGetLoadedClassesResultantClassCount;
static jclass* MockGetLoadedClassesResultantClasses;
static jvmtiError MockGetLoadedClassesReturnValue;
jvmtiError (JNICALL MockGetLoadedClasses) (jvmtiEnv* env,
    jint* classCount,
    jclass** classes) {
	MockGetLoadedClassesCount++;
	*classCount = MockGetLoadedClassesResultantClassCount;
	*classes = MockGetLoadedClassesResultantClasses;
	return MockGetLoadedClassesReturnValue;
}

static const char* MockRecordObjectClassName;
static size_t MockRecordObjectSize;
static int MockPrintCount;

class MockHeapStats: public HeapStats {
public:
   MockHeapStats() {}
   virtual ~MockHeapStats() {}

   void recordObject(const char *className, size_t objectSize) {
     MockRecordObjectClassName = className;
     MockRecordObjectSize = objectSize;
   }
  
   void print(std::ostream& os) const {
   	 MockPrintCount++;
   }
};

class MockHeapStatsFactory: public HeapStatsFactory {
public:
   MockHeapStatsFactory() {}

   virtual ~MockHeapStatsFactory() {}

   HeapStats* create() {
     return new MockHeapStats();
   }
};

static MockHeapStatsFactory* MockHSFactory;

void setup() {
	mockJNIEnv = 0; // FIXME

	mockJvmtiEnvStruct.functions = &mockJvmtiInterface_1_;

	MockGetCapabilitiesCount = 0;
	MockGetCapabilitiesReturnValue = JVMTI_ERROR_NONE;
	((struct jvmtiInterface_1_ *)mockJvmtiEnvStruct.functions)->GetCapabilities = &MockGetCapabilities;
	
	MockAddCapabilitiesCount = 0;
	MockAddCapabilitiesReturnValue = JVMTI_ERROR_NONE;
	((struct jvmtiInterface_1_ *)mockJvmtiEnvStruct.functions)->AddCapabilities = &MockAddCapabilities;

	MockGetLoadedClassesCount = 0;
	MockGetLoadedClassesReturnValue = JVMTI_ERROR_NONE;
	((struct jvmtiInterface_1_ *)mockJvmtiEnvStruct.functions)->GetLoadedClasses = &MockGetLoadedClasses;

	mockJvmtiEnv = &mockJvmtiEnvStruct;

	MockHSFactory = new MockHeapStatsFactory();
	MockPrintCount = 0;
}

void teardown() {
	// placeholder in case setup acquires resources
}

bool testConstructionOk() {
	setup();

	heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv, MockHSFactory);
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
		heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv, MockHSFactory);
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
		heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv, MockHSFactory);
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

// TODO: need to return at least one loaded class and test that
// HeapHistogramAction::tagLoadedClass works correctly, i.e. that
// GetClassSignature and SetTag are called correctly.
//
// TODO: add two more test methods for when GetClassSignature or SetTag fails
bool testHeapStatsPrintOk() {
	setup();
	MockGetLoadedClassesResultantClassCount = 0;

	heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv, MockHSFactory);
	heapHistogramAction->act(mockJNIEnv);
	bool passed = (MockGetLoadedClassesCount == 1) &&
					(MockPrintCount == 1);
    
	if (!passed) {
		fprintf(stdout, "testHeapStatsPrintOk FAILED\n");
	}

	teardown();
	return passed;
}

bool testHeapStatsGetLoadedClassesFailure() {
	setup();
	MockGetLoadedClassesReturnValue = JVMTI_ERROR_ACCESS_DENIED;

	heapHistogramAction = new HeapHistogramAction(mockJvmtiEnv, MockHSFactory);
	
    bool passed = false;
    try {
		heapHistogramAction->act(mockJNIEnv);
    } catch (std::runtime_error *re) {
    	passed = (MockGetCapabilitiesCount == 1) &&
    				(MockAddCapabilitiesCount == 1) &&
    				(MockGetLoadedClassesCount == 1);
	}

	if (!passed) {
		fprintf(stdout, "testHeapStatsGetLoadedClassesFailure FAILED\n");
	}

	teardown();
	return passed;
}

int main() {
	bool result = (testConstructionOk() &&
						testConstructionGetCapabilitiesFailure() &&
						testConstructionAddCapabilitiesFailure() &&
						testHeapStatsPrintOk() &&
						testHeapStatsGetLoadedClassesFailure());
	if (result) {
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else {
    	fprintf(stdout, "FAILURE\n");
    	exit(EXIT_FAILURE);
	}
}