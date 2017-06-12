/*
 * Copyright (c) 2017 the original author or authors.
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

#include <sys/stat.h>
#include <sys/types.h>
#include <iostream>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "heapdumpaction.h"

HeapDumpAction::HeapDumpAction(char* path) {
   if (strlen(path) == 0 || path[0] != '/') {
       char* current = getcwd(NULL, 0);
       char* absPath = new char[strlen(current)+strlen(path)+2];
       strcpy(absPath, current);
       if (absPath[strlen(absPath)-1] != '/') {
           strcat(absPath, "/");
       }
       strcat(absPath, path);
       heapDumpPath = absPath;
       free(current);
   } else {
       heapDumpPath = path;
   }
}

HeapDumpAction::~HeapDumpAction() {
}

// Creates a directory at the specified absolute path with the specified mode.
void mkdir_p(const char* path, mode_t mode) {
    int len = strlen(path);
    char* parent = new char[len+1];
    for (int i = 1; i < len; i++) { // start at 1 since '/' must exist
        if (path[i] == '/') {
            strncpy(parent, path, i);
            parent[i] = '\0';
            mkdir(parent, mode);
        }
    }
}

void HeapDumpAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
	    jclass mfCls = jniEnv->FindClass("java/lang/management/ManagementFactory");
        if (mfCls == NULL) {
            std::cerr << "ERROR: java.lang.management.ManagementFactory class not found\n";
            return;
        }

	    jclass hotSpotDiagnosticMXBeanCls = jniEnv->FindClass("com/sun/management/HotSpotDiagnosticMXBean");
        if (hotSpotDiagnosticMXBeanCls == NULL) {
            std::cerr << "ERROR: com.sun.management.HotSpotDiagnosticMXBean class not found\n";
            return;
        }

        jmethodID getPlatformMXBeanMeth = jniEnv->GetStaticMethodID(mfCls, "getPlatformMXBean", "(Ljava/lang/Class;)Ljava/lang/management/PlatformManagedObject;");
        if (getPlatformMXBeanMeth == NULL) {
            std::cerr << "ERROR: getPlatformMXBean method not found\n";
            return;
        }

        jobject hotSpotDiagnosticMXBean = jniEnv->CallStaticObjectMethod(mfCls, getPlatformMXBeanMeth, hotSpotDiagnosticMXBeanCls);
        if (hotSpotDiagnosticMXBean == NULL) {
            std::cerr << "ERROR: getPlatformMXBean returned null\n";
            return;
        }

        jmethodID dumpHeapMeth = jniEnv->GetMethodID(hotSpotDiagnosticMXBeanCls, "dumpHeap", "(Ljava/lang/String;Z)V");
        if (dumpHeapMeth == NULL) {
            std::cerr << "ERROR: dumpHeap method not found\n";
            return;
        }

        time_t rawNow;
        struct tm* now;

        time(&rawNow);
        now = localtime(&rawNow);

        size_t maxlen = strlen(heapDumpPath)+50;
        char* resolvedHeapDumpPath = new char[maxlen];
        strftime(resolvedHeapDumpPath, maxlen, heapDumpPath, now);
        mkdir_p(resolvedHeapDumpPath, 0700);

        jstring path = jniEnv->NewStringUTF(resolvedHeapDumpPath);

        jniEnv->CallObjectMethod(hotSpotDiagnosticMXBean, dumpHeapMeth, path, JNI_TRUE);
        if (jniEnv->ExceptionCheck()) {
            jthrowable exc = jniEnv->ExceptionOccurred();
            jclass excClass = jniEnv->GetObjectClass(exc);
            jmethodID getMessageMeth = jniEnv->GetMethodID(excClass, "getMessage", "()Ljava/lang/String;");
            jstring message = (jstring)jniEnv->CallObjectMethod(exc, getMessageMeth);
            const char *msgStr = jniEnv->GetStringUTFChars(message, NULL);
            std::cerr << "ERROR: dumpHeap method threw an exception: " << msgStr << "\n";
            jniEnv->ReleaseStringUTFChars(message, msgStr);
            return;
        }
        std::cerr << "\nHeapdump written to " << resolvedHeapDumpPath << "\n";
}
