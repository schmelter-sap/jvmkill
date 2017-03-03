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

#ifndef heaphistogramaction_h
#define heaphistogramaction_h

#include "action.h"
#include "heapstats.h"
#include <jni.h>
#include <jvmti.h>
#include <unordered_map>
using namespace std;

class HeapHistogramAction: public Action
{
public:
   HeapHistogramAction(jvmtiEnv *jvmti, HeapStatsFactory* factory);

   virtual ~HeapHistogramAction();

   void act(JNIEnv* jniEnv, jint resourceExhaustionFlags);

   jint heapReferenceCallback(jvmtiHeapReferenceKind reference_kind, 
     const jvmtiHeapReferenceInfo* reference_info, 
     jlong class_tag, 
     jlong referrer_class_tag, 
     jlong size, 
     jlong* tag_ptr, 
     jlong* referrer_tag_ptr, 
     jint length);
   
private:
   jvmtiEnv* jvmti;
   HeapStatsFactory* heapStatsFactory;
   HeapStats* heapStats;

   // Map from class tag to class signature
   unordered_map<jlong, const char*> taggedClass;
   jlong nextClassTag;

   void printHistogram(JNIEnv* jniEnv, std::ostream *outputStream);
   void tagLoadedClasses(JNIEnv* jniEnv);
   void tagLoadedClass(JNIEnv* jniEnv, jclass& cls);
};

#endif // heaphistogramaction_h
