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
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <iostream>

#include "heaphistogramaction.h"

void printHistogram(jvmtiEnv *jvmti, std::ostream *outputStream) {
	*outputStream << "Histogram goes here";
}

HeapHistogramAction::HeapHistogramAction(jvmtiEnv *jvm) {
	jvmti=jvm;
}

void HeapHistogramAction::act() {
	fprintf(stderr, "Printing Heap Histogram to standard output\n");
	jvmtiCapabilities capabilities;

	/* Get/Add JVMTI capabilities */
	int err = jvmti->GetCapabilities(&capabilities);
    if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: GetCapabilities failed: %d\n", err);
      return;
    }
	capabilities.can_tag_objects = 1;
	capabilities.can_generate_garbage_collection_events = 1;
	capabilities.can_get_source_file_name = 1;
	capabilities.can_get_line_numbers = 1;
	capabilities.can_suspend = 1;
	err = jvmti->AddCapabilities(&capabilities);
	if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: AddCapabilities failed: %d\n", err);
      return;
    }

	printHistogram(jvmti, &(std::cout));
	fprintf(stderr, "Printed Heap Histogram to standard output\n");

}
