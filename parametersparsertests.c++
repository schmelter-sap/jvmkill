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

#include <dlfcn.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "parameters.h"
#include "parametersparser.h"

ParametersParser *parser;

void setup() {
   parser = new ParametersParser();
}

void teardown() {
}


bool testsDefaults() {
   AgentParameters params = parser->parse(strdup(""));
   bool passed = ((params.time_threshold == 1) && (params.count_threshold == 0) && (params.print_heap_histogram == false)
       && params.heap_histogram_max_entries == 100) && (params.print_memory_usage == true);
   if (!passed) {
      fprintf(stdout, "testsDefaults FAILED\n");
   }
   return passed;
}

bool testsParsesTimeThreshold() {
   AgentParameters params = parser->parse(strdup("time=99"));
   bool passed = (params.time_threshold == 99);
   if (!passed) {
      fprintf(stdout, "testsParsesTimeThreshold FAILED\n");
   }
   return passed;
}

bool testsParsesDefaultTimeThreshold() {
   AgentParameters params = parser->parse(strdup("time="));
   bool passed = (params.time_threshold == 1);
   if (!passed) {
      fprintf(stdout, "testsParsesDefaultTimeThreshold FAILED\n");
   }
   return passed;
}

bool testsParsesCountThreshold() {
   AgentParameters params = parser->parse(strdup("count=88"));
   bool passed = (params.count_threshold == 88);
   if (!passed) {
      fprintf(stdout, "testsParsesCountThreshold FAILED\n");
   }
   return passed;
}

bool testsParsesDefaultCountThreshold() {
   AgentParameters params = parser->parse(strdup("count="));
   bool passed = (params.count_threshold == 0);
   if (!passed) {
      fprintf(stdout, "testsParsesDefaultCountThreshold FAILED\n");
   }
   return passed;
}

bool testsParsesPrintHeapHistogramOn() {
   AgentParameters params = parser->parse(strdup("printHeapHistogram=1"));
   bool passed = (params.print_heap_histogram == true);
   if (!passed) {
      fprintf(stdout, "testsParsesPrintHeapHistogramOn FAILED\n");
   }
   return passed;
}

bool testsParsesPrintHeapHistogramOff() {
   AgentParameters params = parser->parse(strdup("printHeapHistogram=0"));
   bool passed = (params.print_heap_histogram == false);
   if (!passed) {
      fprintf(stdout, "testsParsesPrintHeapHistogramOff FAILED\n");
   }
   return passed;
}

bool testsParsesDefaultPrintHeapHistogram() {
   AgentParameters params = parser->parse(strdup("printHeapHistogram="));
   bool passed = (params.print_heap_histogram == false);
   if (!passed) {
      fprintf(stdout, "testsParsesDefaultPrintHeapHistogram FAILED\n");
   }
   return passed;
}

bool testsParsesHeapHistogramMaxEntries() {
   AgentParameters params = parser->parse(strdup("heapHistogramMaxEntries=200"));
   bool passed = (params.heap_histogram_max_entries == 200);
   if (!passed) {
      fprintf(stdout, "testsParsesHeapHistogramMaxEntries FAILED\n");
   }
   return passed;
}

bool testsParsesHeapHistogramMaxEntriesUnlimited() {
   AgentParameters params = parser->parse(strdup("heapHistogramMaxEntries=0"));
   bool passed = (params.heap_histogram_max_entries == 0);
   if (!passed) {
      fprintf(stdout, "testsParsesHeapHistogramMaxEntriesUnlimited FAILED\n");
   }
   return passed;
}

bool testsParsesDefaultHeapHistogramMaxEntries() {
   AgentParameters params = parser->parse(strdup("heapHistogramMaxEntries="));
   bool passed = (params.heap_histogram_max_entries == 100);
   if (!passed) {
      fprintf(stdout, "testsParsesDefaultHeapHistogramMaxEntries FAILED\n");
   }
   return passed;
}

bool testsParsesPrintMemoryUsageOn() {
   AgentParameters params = parser->parse(strdup("printMemoryUsage=1"));
   bool passed = (params.print_memory_usage == true);
   if (!passed) {
      fprintf(stdout, "testsParsesPrintMemoryUsageOn FAILED\n");
   }
   return passed;
}

bool testsParsesPrintMemoryUsageOff() {
   AgentParameters params = parser->parse(strdup("printMemoryUsage=0"));
   bool passed = (params.print_memory_usage == false);
   if (!passed) {
      fprintf(stdout, "testsParsesPrintMemoryUsageOff FAILED\n");
   }
   return passed;
}

bool testsParsesDefaultPrintMemoryUsage() {
   AgentParameters params = parser->parse(strdup("printMemoryUsage="));
   bool passed = (params.print_memory_usage == true);
   if (!passed) {
      fprintf(stdout, "testsParsesDefaultPrintMemoryUsage FAILED\n");
   }
   return passed;
}

int main() {
	setup();
	bool result = testsDefaults() &&
	   testsParsesTimeThreshold() && testsParsesDefaultTimeThreshold() &&
	   testsParsesCountThreshold() && testsParsesDefaultCountThreshold() &&
	   testsParsesPrintHeapHistogramOn() && testsParsesPrintHeapHistogramOff() && testsParsesDefaultPrintHeapHistogram() &&
	   testsParsesHeapHistogramMaxEntries() && testsParsesHeapHistogramMaxEntriesUnlimited() && testsParsesDefaultHeapHistogramMaxEntries();
	teardown();
	if (result) {
       fprintf(stdout, "SUCCESS\n");
	   exit(EXIT_SUCCESS);
	}
	else {
       fprintf(stdout, "FAILURE\n");
       exit(EXIT_FAILURE);
	}
}
