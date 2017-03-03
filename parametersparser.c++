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

#include <sys/types.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "parametersparser.h"

const int DEFAULT_COUNT_THRESHOLD = 0;
const int DEFAULT_TIME_THRESHOLD = 1;
const int DEFAULT_PRINT_HEAP_HISTOGRAM = 0;
const int DEFAULT_HEAP_HISTOGRAM_MAX_ENTRIES = 100;
const int DEFAULT_PRINT_MEMORY_USAGE = 1;

enum {
    TIME_OPT = 0,
    COUNT_OPT,
    PRINT_HEAP_HISTOGRAM_OPT,
    HEAP_HISTOGRAM_MAX_ENTRIES_OPT,
    PRINT_MEMORY_USAGE_OPT,
    THE_END
};

char *tokens[] = {
    [TIME_OPT] = strdup("time"),
    [COUNT_OPT] = strdup("count"),
    [PRINT_HEAP_HISTOGRAM_OPT] = strdup("printHeapHistogram"),
    [HEAP_HISTOGRAM_MAX_ENTRIES_OPT] = strdup("heapHistogramMaxEntries"),
    [PRINT_MEMORY_USAGE_OPT] = strdup("printMemoryUsage"),
    [THE_END] = NULL
};

void checkValueProvided(char *value, int option) {
   if (value == NULL) {
      fprintf(stderr, "Suboption '%s=<value>' did not have a value\n", tokens[option]);
      abort();
   }
}

ParametersParser::ParametersParser() {
}

AgentParameters ParametersParser::parse(char *options) {
  AgentParameters result;
  //sets defaults
  result.count_threshold = DEFAULT_COUNT_THRESHOLD;
  result.time_threshold = DEFAULT_TIME_THRESHOLD;
  result.print_heap_histogram = DEFAULT_PRINT_HEAP_HISTOGRAM;
  result.heap_histogram_max_entries = DEFAULT_HEAP_HISTOGRAM_MAX_ENTRIES;
  result.print_memory_usage = DEFAULT_PRINT_MEMORY_USAGE;

  if (options != NULL) {
     // Copy input options since getsubopt modifies its input
     char *subopts = new char[strlen(options) + 1];
     strcpy(subopts, options);

     char *value;

     while (*subopts != '\0') {
        switch (getsubopt(&subopts, tokens, &value)) {
           case COUNT_OPT:
              checkValueProvided(value, COUNT_OPT);
              result.count_threshold = (strlen(value) == 0) ? DEFAULT_COUNT_THRESHOLD : atoi(value);
              break;

           case TIME_OPT:
              checkValueProvided(value, TIME_OPT);
              result.time_threshold = (strlen(value) == 0) ? DEFAULT_TIME_THRESHOLD : atoi(value);
              break;

          case PRINT_HEAP_HISTOGRAM_OPT:
              checkValueProvided(value, PRINT_HEAP_HISTOGRAM_OPT);
              result.print_heap_histogram = (strlen(value) == 0) ? DEFAULT_PRINT_HEAP_HISTOGRAM : atoi(value);
              break;

          case HEAP_HISTOGRAM_MAX_ENTRIES_OPT:
              checkValueProvided(value, HEAP_HISTOGRAM_MAX_ENTRIES_OPT);
              result.heap_histogram_max_entries = (strlen(value) == 0) ? DEFAULT_HEAP_HISTOGRAM_MAX_ENTRIES : atoi(value);
              break;

          case PRINT_MEMORY_USAGE_OPT:
              checkValueProvided(value, PRINT_MEMORY_USAGE_OPT);
              result.print_memory_usage = (strlen(value) == 0) ? DEFAULT_PRINT_MEMORY_USAGE : atoi(value);
              break;

          default:
              // Print the unrecognised option name and value.
              // Note: Darwin's getsubopt omits the option name and equals sign from value in this case.
              fprintf(stderr, "Unknown suboption '%s'\n", value);
              break;
        }
     }
  }
  return result;
}
