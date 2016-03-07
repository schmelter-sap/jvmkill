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

#ifdef __MACH__
#include <mach/mach_time.h>
#define CLOCK_MONOTONIC 0
int clock_gettime(int clk_id, struct timespec *t){
    mach_timebase_info_data_t timebase;
    mach_timebase_info(&timebase);
    uint64_t time;
    time = mach_absolute_time();
    t->tv_sec = ((double)time * (double)timebase.numer)/((double)timebase.denom * 1e9);
    t->tv_nsec = ((double)time * (double)timebase.numer)/((double)timebase.denom);
    return 0;
}
#else
#include <time.h>
#endif

#include "jvmkill.h"

enum {
    TIME_OPT = 0,
    COUNT_OPT,
    THE_END
};
 
char *tokens[] = {
    [TIME_OPT] = strdup("time"),
    [COUNT_OPT] = strdup("count"),
    [THE_END] = NULL
};

static jrawMonitorID monitorID;
static long *events; 
static int eventIndex = 0;
static struct Configuration configuration;

void setSignal(int signal) {
   configuration.signal = signal;
}

static long getTimeMillis() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (ts.tv_sec*1000)+(ts.tv_nsec/1000000);
}

static long getMillisLimit() {
   return getTimeMillis()-configuration.time_threshold*1000;
}

static void addEvent() {
   events[eventIndex]=getTimeMillis();
   if (++eventIndex>=configuration.count_threshold) {
      eventIndex=0;
   }
}

static int countEvents() {
   long millisLimit = getMillisLimit();
   int count = 0;
   for (int i=0;i<configuration.count_threshold;i++) {
      if (events[i] != 0 && events[i]>=millisLimit) 
     	   count++;
   }
   return count;
}

void resourceExhausted(
      jvmtiEnv *jvmti_env,
      JNIEnv *jni_env,
      jint flags,
      const void *reserved,
      const char *description) {
   jvmtiError err;

   err = jvmti_env->RawMonitorEnter(monitorID);
   if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: RawMonitorEnter failed: %d\n", err);
      return;
   }

   int eventCount = countEvents();
   fprintf(stderr, "ResourceExhausted! (%d/%d)\n", eventCount+1, configuration.count_threshold);
   //eventCount was already on threshold before adding the current one
   if (eventCount == configuration.count_threshold) {
        fprintf(stderr, "killing current process\n");
        kill(getpid(), configuration.signal);
   }
   addEvent(); // FIXME: move up?

   err = jvmti_env->RawMonitorExit(monitorID);
   if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: RawMonitorExit failed: %d\n", err);
   }
}

int setCallbacks(jvmtiEnv *jvmti) {
   jvmtiError err;

   err = jvmti->CreateRawMonitor("jvmkillMonitor", &monitorID);
   if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: CreateRawMonitor failed: %d\n", err);
      return JNI_ERR;
   }

   jvmtiEventCallbacks callbacks;
   memset(&callbacks, 0, sizeof(callbacks));

   callbacks.ResourceExhausted = &resourceExhausted;

   err = jvmti->SetEventCallbacks(&callbacks, sizeof(callbacks));
   if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: SetEventCallbacks failed: %d\n", err);
      return JNI_ERR;
   }

   err = jvmti->SetEventNotificationMode(JVMTI_ENABLE, JVMTI_EVENT_RESOURCE_EXHAUSTED, NULL);
   if (err != JVMTI_ERROR_NONE) {
      fprintf(stderr, "ERROR: SetEventNotificationMode failed: %d\n", err);
      return JNI_ERR;
   }
   
   return JNI_OK;
}

int getCount_Threshold() {
   return configuration.count_threshold;
}

int getTime_Threshold() {
   return configuration.time_threshold;
}

void setParameters(char *options) {
   char *subopts;
   char *value;

   //sets defaults
   configuration.count_threshold = 0;
   configuration.time_threshold = 1;

   if (NULL == options)
       return;

   subopts = options;
   while (*subopts != '\0') {
      switch (getsubopt (&subopts, tokens, &value)) {
         case COUNT_OPT:
            if (value == NULL)
               abort ();
            configuration.count_threshold = atoi (value);
            break;
         case TIME_OPT:
            if (value == NULL)
               abort ();
            configuration.time_threshold = atoi (value);
            break;
         default:
            /* Unknown suboption. */
            fprintf (stderr, "Unknown suboption '%s'\n", value);
            break;
      }
   }
   events = new long[configuration.count_threshold];
   //prefill with a safe value
   for (int i=0;i<configuration.count_threshold;i++) {
          events[i]=0;
   }
}

JNIEXPORT jint JNICALL
Agent_OnLoad(JavaVM *vm, char *options, void *reserved)
{
   jvmtiEnv *jvmti;
   
   configuration.signal = SIGKILL;

   jint rc = vm->GetEnv((void **) &jvmti, JVMTI_VERSION);
   if (rc != JNI_OK) {
      fprintf(stderr, "ERROR: GetEnv failed: %d\n", rc);
      return JNI_ERR;
   }
   setParameters(options);	
   return setCallbacks(jvmti);
}


