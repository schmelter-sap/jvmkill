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
#include <stdio.h>
#include <stdlib.h>

#include <jvmti.h>

#include "agentcontroller.h"
#include "parameters.h"
#include "parametersparser.h"
#include "heuristic.h"
#include "threshold.h"
#include "killaction.h"
#include "poolstatsaction.h"
#include "heaphistogramaction.h"
#include "heapstatshashtable.h"
#include "action.h"

static int ActionRunCounter = 0;

static int MockPrintHeapActionCount = 0;
static int MockKillActionRunOrder = -1;
static int MockPoolStatsActionRunOrder = -1;
static jint MockPoolStatsActionFlags = 0;
static int MockPrintHeapActionRunOrder = -1;
static jint MockPrintHeapActionFlags = 0;
static int MockPoolStatsActionCount = 0;
static int MockKillActionCount = 0;
static int MockThresholdEventCount = 0;
AgentController* agentController;
jvmtiEnv* jvm;
JNIEnv* mockJNIEnv;
/************************************************
 *  mocks
 ************************************************/
HeapHistogramAction::HeapHistogramAction(jvmtiEnv* jvm, HeapStatsFactory* factory) {
	MockPrintHeapActionCount++;
}
HeapHistogramAction::~HeapHistogramAction() {
}
void HeapHistogramAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
	MockPrintHeapActionRunOrder = ActionRunCounter++;
	MockPrintHeapActionFlags = resourceExhaustionFlags;
}

//PoolStatsAction

PoolStatsAction::PoolStatsAction() {
	MockPoolStatsActionCount++;
}
PoolStatsAction::~PoolStatsAction() {
}
void PoolStatsAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
	MockPoolStatsActionRunOrder = ActionRunCounter++;
	MockPoolStatsActionFlags = resourceExhaustionFlags;
}

//KillAction

KillAction::KillAction() {
	MockKillActionCount++;
}
void KillAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
	MockKillActionRunOrder = ActionRunCounter++;
}

//ParameterParser

ParametersParser::ParametersParser() {
}

AgentParameters ParametersParser::parse(char *options) {
	struct AgentParameters agentParameters;
	return agentParameters;
}

//Threshold
Threshold::Threshold(AgentParameters param) {
}

long Threshold::getMillisLimit() {
	return 0;
}

void Threshold::addEvent() {
}

int Threshold::countEvents() {
	return 0;
}

bool Threshold::onOOM() {
   return ++MockThresholdEventCount > 1;
}

//HeapStatsHashtable
HeapStatsHashtable::HeapStatsHashtable(int maxEntries) {}

HeapStatsHashtable::~HeapStatsHashtable() {}

void HeapStatsHashtable::recordObject(const char *className, size_t objectSize) {}
  
void HeapStatsHashtable::print(std::ostream& os) const {}

//end of mocks

void setup() {
	agentController = new AgentController(NULL);
	mockJNIEnv = 0;
    ActionRunCounter = 0;

    MockPrintHeapActionCount = 0;
    MockKillActionRunOrder = -1;
    MockPoolStatsActionRunOrder = -1;
    MockPoolStatsActionFlags = 0;
    MockPrintHeapActionRunOrder = -1;
    MockPrintHeapActionFlags = 0;
    MockPoolStatsActionCount = 0;
    MockKillActionCount = 0;
    MockThresholdEventCount = 0;
}

void teardown() {
	delete(agentController);
}

bool testAlwaysAddsKillAction() {
	setup();
	AgentParameters params;
	params.print_heap_histogram = true;
	agentController->setParameters(params);
	bool passed = (MockKillActionCount == 1);
    if (!passed) {
        fprintf(stdout, "testAlwaysAddsKillAction FAILED\n");
    }
	teardown();
	return passed;
}


bool testDoesNotAddHeapActionWhenOff() {
	setup();
	AgentParameters params;
	params.print_heap_histogram = false;
	agentController->setParameters(params);
    bool passed = (MockPrintHeapActionCount == 0);
    if (!passed) {
        fprintf(stdout, "testDoesNotAddHeapActionWhenOff FAILED\n");
    }
	teardown();
	return passed;
}

bool testAddsHeapActionWhenOn() {
	setup();
	AgentParameters params;
	params.print_heap_histogram = true;
	agentController->setParameters(params);
	bool passed = (MockPrintHeapActionCount == 1);
    if (!passed) {
        fprintf(stdout, "testAddsHeapActionWhenOn FAILED\n");
    }
	teardown();
	return passed;
}

bool testDoesNotAddPoolStatsActionWhenOff() {
	setup();
	AgentParameters params;
	params.print_memory_usage = false;
	agentController->setParameters(params);
    bool passed = (MockPoolStatsActionCount == 0);
    if (!passed) {
        fprintf(stdout, "testDoesNotAddPoolStatsActionWhenOff FAILED\n");
    }
	teardown();
	return passed;
}

bool testAddsPoolStatsActionWhenOn() {
	setup();
	AgentParameters params;
	params.print_memory_usage = true;
	agentController->setParameters(params);
	bool passed = (MockPoolStatsActionCount == 1);
    if (!passed) {
        fprintf(stdout, "testAddsPoolStatsActionWhenOn FAILED\n");
    }
	teardown();
	return passed;
}

bool testRunsAllActionsInCorrectOrderOnOOM() {
	setup();
	AgentParameters params;
	params.print_heap_histogram = true;
	params.print_memory_usage = true;
	agentController->setParameters(params);

	//MockThreshold returns true for OOM on second attempt, therefore should not
	//run actions on first call
	agentController->onOOM(mockJNIEnv, 5);
	bool firstAssertions = ((MockPrintHeapActionRunOrder == -1) &&
	    (MockPoolStatsActionRunOrder == -1) &&
	    (MockKillActionRunOrder == -1) &&
		(MockThresholdEventCount == 1));

	agentController->onOOM(mockJNIEnv, 5);
    bool passed = ((MockPrintHeapActionRunOrder == 0) &&
        (MockPrintHeapActionFlags == 5) &&
        (MockPoolStatsActionRunOrder == 1) &&
        (MockPoolStatsActionFlags == 5) &&
	    (MockKillActionRunOrder == 2) &&
		(MockThresholdEventCount > 1) &&
		(firstAssertions));
    if (!passed) {
        fprintf(stdout, "testRunsAllActionsInCorrectOrderOnOOM FAILED\n");
    }
	teardown();
    return passed;
}

bool testRunsOnlyEnabledActionsOnOOM() {
	setup();
	AgentParameters params;
	params.print_heap_histogram = false;
	params.print_memory_usage = false;
	agentController->setParameters(params);

	//MockThreshold returns true for OOM on second attempt, therefore should not
	//run actions on first call
	agentController->onOOM(mockJNIEnv, 5);
	bool firstAssertions = ((MockPrintHeapActionRunOrder == -1) &&
	    (MockPoolStatsActionRunOrder == -1) &&
	    (MockKillActionRunOrder == -1) &&
		(MockThresholdEventCount == 1));

	agentController->onOOM(mockJNIEnv, 5);
    bool passed = ((MockPrintHeapActionRunOrder == -1) &&
        (MockPoolStatsActionRunOrder == -1) &&
	    (MockKillActionRunOrder == 0) &&
		(MockThresholdEventCount > 1) &&
		(firstAssertions));
    if (!passed) {
        fprintf(stdout, "testRunsOnlyEnabledActionsOnOOM FAILED\n");
    }
	teardown();
    return passed;
}


int main() {
	bool result = (testDoesNotAddHeapActionWhenOff() &&
				   testAddsHeapActionWhenOn() &&
				   testDoesNotAddPoolStatsActionWhenOff() &&
				   testAddsPoolStatsActionWhenOn() &&
				   testRunsAllActionsInCorrectOrderOnOOM() &&
				   testRunsOnlyEnabledActionsOnOOM());
	if (result) {
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else {
    	fprintf(stdout, "FAILURE\n");
    	exit(EXIT_FAILURE);
	}
}
