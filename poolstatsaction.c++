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

#include <sys/types.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <string>
#include <unistd.h>
#include <iostream>
#include <sstream>
#include <jvmti.h>
#include <chrono>
#include <thread>

#include "poolstatsaction.h"

PoolStatsAction::PoolStatsAction() {
}

PoolStatsAction::~PoolStatsAction() {
}


void PoolStatsAction::act(JNIEnv* jniEnv, jint resourceExhaustionFlags) {
    // Do not attempt to obtain pool stats on thread exhaustion as this fails abruptly.
    if ((resourceExhaustionFlags & JVMTI_RESOURCE_EXHAUSTED_THREADS) == JVMTI_RESOURCE_EXHAUSTED_THREADS) {
        std::cout << "\nThe VM was unable to create a thread. In these circumstances, memory usage statistics cannot be determined." << std::endl;
        return;
    }

    jclass mfCls = jniEnv->FindClass("java/lang/management/ManagementFactory");
    if (mfCls == NULL) {
        std::cerr << "ERROR: java.lang.management.ManagementFactory class not found\n";
        return;
    }

    jmethodID getMemMXBeanMeth = jniEnv->GetStaticMethodID(mfCls, "getMemoryMXBean", "()Ljava/lang/management/MemoryMXBean;");
    if (getMemMXBeanMeth == NULL) {
        std::cerr << "ERROR: getMemoryMXBean method not found\n";
        return;
    }

    jobject memMXBean = jniEnv->CallStaticObjectMethod(mfCls, getMemMXBeanMeth);
    if (memMXBean == NULL) {
        std::cerr << "ERROR: getMemoryMXBean returned null\n";
        return;
    }

    jclass memMXBeanCls = jniEnv->GetObjectClass(memMXBean);
    if (memMXBeanCls == NULL) {
        std::cerr << "ERROR: java.lang.management.MemoryMXBean class not found\n";
        return;
    }

    jmethodID heapMemoryUsageMeth = jniEnv->GetMethodID(memMXBeanCls, "getHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;");
    if (heapMemoryUsageMeth == NULL) {
        std::cerr << "ERROR: getHeapMemoryUsage method not found\n";
        return;
    }

    jobject heapUsage = jniEnv->CallObjectMethod(memMXBean, heapMemoryUsageMeth);
    if (heapUsage == NULL) {
        std::cerr << "ERROR: getHeapMemoryUsage returned null\n";
        return;
    }

    jmethodID nonHeapMemoryUsageMeth = jniEnv->GetMethodID(memMXBeanCls, "getNonHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;");
    if (nonHeapMemoryUsageMeth == NULL) {
        std::cerr << "ERROR: getNonHeapMemoryUsage method not found\n";
        return;
    }

    jobject nonHeapUsage = jniEnv->CallObjectMethod(memMXBean, nonHeapMemoryUsageMeth);
    if (nonHeapUsage == NULL) {
        std::cerr << "ERROR: getNonHeapMemoryUsage returned null\n";
        return;
    }

    std::string heapUsageStats = usageStats(jniEnv, heapUsage);
    if (heapUsageStats == "FAILED") {
      return;
    }

    std::string nonHeapUsageStats = usageStats(jniEnv, nonHeapUsage);
    if (nonHeapUsageStats == "FAILED") {
      return;
    }

    std::chrono::milliseconds timespan(1);

    std::cout << "\nMemory usage:" << std::endl <<
        "   Heap memory: " << heapUsageStats << std::endl <<
        "   Non-heap memory: " << nonHeapUsageStats << std::endl;

    std::cout << "\nMemory pool usage:" << std::endl;

    // Reduce the risk of loggregator missing some entries.
    std::this_thread::sleep_for(timespan);

    jmethodID getMemPoolMXBeansMeth = jniEnv->GetStaticMethodID(mfCls, "getMemoryPoolMXBeans", "()Ljava/util/List;");
    if (getMemPoolMXBeansMeth == NULL) {
        std::cerr << "ERROR: getMemoryPoolMXBeans method not found\n";
        return;
    }

    jobject memPoolMXBeans = jniEnv->CallStaticObjectMethod(mfCls, getMemPoolMXBeansMeth);
    if (memPoolMXBeans == NULL) {
        std::cerr << "ERROR: getMemoryPoolMXBeans returned null\n";
        return;
    }


    jclass listCls = jniEnv->FindClass("java/util/List");
    if (listCls == NULL) {
        std::cerr << "ERROR: java.util.List class not found\n";
        return;
    }

    jmethodID sizeMeth = jniEnv->GetMethodID(listCls, "size", "()I");
    if (sizeMeth == NULL) {
        std::cerr << "ERROR: size method not found\n";
        return;
    }

    jmethodID getMeth = jniEnv->GetMethodID(listCls, "get", "(I)Ljava/lang/Object;");
    if (getMeth == NULL) {
        std::cerr << "ERROR: get method not found\n";
        return;
    }
    
    jclass memPoolMXBeanCls = jniEnv->FindClass("java/lang/management/MemoryPoolMXBean");
    if (memPoolMXBeanCls == NULL) {
        std::cerr << "ERROR: java.lang.management.MemoryPoolMXBean class not found\n";
        return;
    }

    jmethodID getNameMeth = jniEnv->GetMethodID(memPoolMXBeanCls, "getName", "()Ljava/lang/String;");
    if (getNameMeth == NULL) {
        std::cerr << "ERROR: getName method not found\n";
        return;
    }

    jmethodID getUsageMeth = jniEnv->GetMethodID(memPoolMXBeanCls, "getUsage", "()Ljava/lang/management/MemoryUsage;");
    if (getUsageMeth == NULL) {
        std::cerr << "ERROR: getUsage method not found\n";
        return;
    }

    jint size = jniEnv->CallIntMethod(memPoolMXBeans, sizeMeth);

    for (jint i = 0; i < size; i++) {
        jobject poolMXBean = jniEnv->CallObjectMethod(memPoolMXBeans, getMeth, i);
        if (poolMXBean == NULL) {
            std::cerr << "ERROR: get returned null\n";
            return;
        }

        jstring nameObj = (jstring)jniEnv->CallObjectMethod(poolMXBean, getNameMeth);
        if (nonHeapUsage == NULL) {
            std::cerr << "ERROR: getName returned null\n";
            return;
        }

        jobject usage = jniEnv->CallObjectMethod(poolMXBean, getUsageMeth);
        if (nonHeapUsage == NULL) {
            std::cerr << "ERROR: getUsage returned null\n";
            return;
        }

        const char* name = jniEnv->GetStringUTFChars(nameObj, NULL);

        std::cout << "   " << name << ": " << usageStats(jniEnv, usage) << std::endl;

        // Reduce the risk of loggregator missing some entries.
        std::this_thread::sleep_for(timespan);

        jniEnv->ReleaseStringUTFChars(nameObj, name);
    }
}

std::string PoolStatsAction::usageStats(JNIEnv* jniEnv, jobject usage) {
    jclass memoryUsageCls = jniEnv->GetObjectClass(usage);
    if (memoryUsageCls == NULL) {
        std::cerr << "ERROR: java.lang.management.MemoryUsage class not found\n";
        return "FAILED";
    }

    jmethodID initMeth = jniEnv->GetMethodID(memoryUsageCls, "getInit", "()J");
    if (initMeth == NULL) {
        std::cerr << "ERROR: getInit method not found\n";
        return "FAILED";
    }

    jmethodID usedMeth = jniEnv->GetMethodID(memoryUsageCls, "getUsed", "()J");
    if (usedMeth == NULL) {
        std::cerr << "ERROR: getUsed method not found\n";
        return "FAILED";
    }

    jmethodID committedMeth = jniEnv->GetMethodID(memoryUsageCls, "getCommitted", "()J");
    if (committedMeth == NULL) {
        std::cerr << "ERROR: getCommitted method not found\n";
        return "FAILED";
    }

    jmethodID maxMeth = jniEnv->GetMethodID(memoryUsageCls, "getMax", "()J");
    if (maxMeth == NULL) {
        std::cerr << "ERROR: getMax method not found\n";
        return "FAILED";
    }

    std::ostringstream out;

    out << "init " << jniEnv->CallLongMethod(usage, initMeth) <<
        ", used " << jniEnv->CallLongMethod(usage, usedMeth) <<
        ", committed " << jniEnv->CallLongMethod(usage, committedMeth) <<
        ", max " << jniEnv->CallLongMethod(usage, maxMeth);

    return out.str();
}
