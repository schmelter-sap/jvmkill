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

#include <limits.h>
#include <sys/types.h>
#include <signal.h>
#include <stdexcept>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <iostream>

#include "heaphistogramaction.h"
#include "heapstats.h"

// Pick a suitable object tag mask greater than tags used to tag classes.
const jlong TAG_VISITED_MASK = 1 << 31;

jint (JNICALL heapRefCallback)(jvmtiHeapReferenceKind reference_kind, 
     const jvmtiHeapReferenceInfo* reference_info, 
     jlong class_tag, 
     jlong referrer_class_tag, 
     jlong size, 
     jlong* tag_ptr, 
     jlong* referrer_tag_ptr, 
     jint length, 
     void* user_data) {
	return ((HeapHistogramAction*)user_data)->heapReferenceCallback(reference_kind,
		reference_info,
		class_tag,
		referrer_class_tag,
		size,
		tag_ptr,
		referrer_tag_ptr,
		length);
}

void HeapHistogramAction::printHistogram(JNIEnv* jniEnv, std::ostream *outputStream) {
	heapStats = heapStatsFactory->create();

	// Tag all loaded classes so we can determine each object's class signature during heap traversal.
	tagLoadedClasses(jniEnv);

	// Traverse the live heap and add objects to the heap stats.
	jvmtiHeapCallbacks callbacks = {};
	callbacks.heap_reference_callback = &heapRefCallback;

	jvmtiError err = jvmti->FollowReferences(0, NULL, NULL, &callbacks, this);
	if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: FollowReferences failed: %d\n", err);
		throw new std::runtime_error("FollowReferences failed");
    }

    // Print the histogram.
	heapStats->print(*outputStream);

	delete heapStats;
	heapStats = NULL;
}

void HeapHistogramAction::tagLoadedClasses(JNIEnv* jniEnv) {
	jint classCount;
	jclass* classes;
	jvmtiError err = jvmti->GetLoadedClasses(&classCount, &classes);
	if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: GetLoadedClasses failed: %d\n", err);
		throw new std::runtime_error("GetLoadedClasses failed");
    }

    for (int i = 0; i < classCount; i++) {
		tagLoadedClass(jniEnv, classes[i]);
    }
}

void HeapHistogramAction::tagLoadedClass(JNIEnv* jniEnv, jclass& cls) {
	char * classSignature;
	jvmtiError err = jvmti->GetClassSignature(cls, &classSignature, 0);
    if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: GetClassSignature failed: %d\n", err);
		throw new std::runtime_error("GetClassSignature failed");
    }

    nextClassTag++;

	err = jvmti->SetTag(cls, nextClassTag);
	if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: SetTag failed: %d\n", err);
		throw new std::runtime_error("SetTag failed");
    }

    taggedClass[nextClassTag] = strdup(classSignature); // Freed in destructor

	jvmti->Deallocate((unsigned char *)classSignature); // Ignore return value
}

jint HeapHistogramAction::heapReferenceCallback(jvmtiHeapReferenceKind reference_kind, 
     const jvmtiHeapReferenceInfo* reference_info, 
     jlong class_tag, 
     jlong referrer_class_tag, 
     jlong size, 
     jlong* tag_ptr, 
     jlong* referrer_tag_ptr, 
     jint length) {
	if (*tag_ptr & TAG_VISITED_MASK) {
		return 0;
	}

	// For each object encountered, tag it so we can avoid visiting it again
	// noting that the histogram is computed at most once in the lifetime of a JVM
	*tag_ptr |= TAG_VISITED_MASK;

	// Add the object to the heap stats along with its class signature.
 	jlong unmaskedClassTag = class_tag & ~TAG_VISITED_MASK;
 	if (taggedClass.find(unmaskedClassTag) != taggedClass.end()) {
		heapStats->recordObject(taggedClass[unmaskedClassTag], size);
	}

	return JVMTI_VISIT_OBJECTS;
}

HeapHistogramAction::HeapHistogramAction(jvmtiEnv *jvm, HeapStatsFactory* factory) {
	jvmtiCapabilities capabilities;

	/* Get/Add JVMTI capabilities */
	int err = jvm->GetCapabilities(&capabilities);
	if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: GetCapabilities failed: %d\n", err);
		throw new std::runtime_error("GetCapabilities failed");
    }

	capabilities.can_tag_objects = 1;

	err = jvm->AddCapabilities(&capabilities);
	if (err != JVMTI_ERROR_NONE) {
		fprintf(stderr, "ERROR: AddCapabilities failed: %d\n", err);
		throw new std::runtime_error("AddCapabilities failed");
    }

	jvmti = jvm;
	heapStatsFactory = factory;
	nextClassTag = 0;
}

HeapHistogramAction::~HeapHistogramAction() {
	// free all values in taggedClass
	for (auto it = taggedClass.begin(); it != taggedClass.end(); ++it) {
		delete[] it->second;
	}
}

void HeapHistogramAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
	printHistogram(jniEnv, &(std::cout));
}
