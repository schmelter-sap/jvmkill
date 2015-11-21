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
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include "jvmkill.h"

static void *handle;
static char *parameters;
static void (*setSignalFn)(int signal);
static void (*setParametersFn)(char *options);
static int (*getTime_ThresholdFn)();
static int (*getCount_ThresholdFn)();
static void (*resourceExhaustedFn)(
      jvmtiEnv *jvmti_env,
      JNIEnv *jni_env,
      jint flags,
      const void *reserved,
      const char *description);

int sigQuit_sent = 0;
int moddedSignal = SIGUSR1;

void sig_handler(int signo) {
	if (signo == moddedSignal)
		sigQuit_sent = 1;
}

void* printDlError(void *handle) {
 	if (!handle) {
        	fprintf(stderr, "%s\n", dlerror());
        	exit(EXIT_FAILURE);
    	}
	else
		return handle;
}

//loads agent library and fetches external function handles
void setup() {
        handle = printDlError(dlopen("libjvmkill.so", RTLD_LAZY));
	parameters = malloc(sizeof(char) * 1024);
	if (signal(moddedSignal, sig_handler) == SIG_ERR)
        	exit(EXIT_FAILURE);

	resourceExhaustedFn = printDlError(dlsym(handle, "resourceExhausted"));
	setParametersFn = printDlError(dlsym(handle, "setParameters"));
	getTime_ThresholdFn = printDlError(dlsym(handle, "getTime_Threshold"));
	getCount_ThresholdFn = printDlError(dlsym(handle, "getCount_Threshold"));

	setSignalFn = printDlError(dlsym(handle, "setSignal"));
	setSignalFn(moddedSignal);
}

//unloads agent library
void teardown() {
	dlclose(handle);
	free(parameters);
}

//tests agent fires signal if threshold is exceeded
int testSendsSignalIfThresholdExceeded() {
	sigQuit_sent=0;
	snprintf(parameters, 1024, "%s", "time=3,count=2"); 
	setParametersFn(parameters);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	if (!sigQuit_sent)
           fprintf(stdout, "testSendsSignalIfThresholdExceeded FAILED\n");
	return sigQuit_sent;
}

//tests agent doesnt fire signal if threshold is reached but not exceeded
int testDoesntSendSignalIfThresholdNotExceeded() {
	sigQuit_sent=0;
	snprintf(parameters, 1024, "%s", "time=3,count=5");
	setParametersFn(parameters);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	sleep(6); //waits for counter to "zero" 
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	resourceExhaustedFn(NULL,NULL,5,NULL,NULL);
	if (sigQuit_sent)
           fprintf(stdout, "testDoesntSendSignalIfThresholdNotExceeded FAILED\n");
	return !sigQuit_sent;

}

//tests agent parameters parsing
int testSetsUpParameters() {
	snprintf(parameters, 1024, "%s", "time=10,count=5"); 
	setParametersFn(parameters);
	return ((getTime_ThresholdFn() == 10) && (getCount_ThresholdFn() == 5));
}

//tests agent default values and parameters parsing when no parameters are provided 
int testSetsUpNoParameters() {
	snprintf(parameters, 1024, "%s", ""); 
	setParametersFn(parameters);
	return (getCount_ThresholdFn() == 0 && (getTime_ThresholdFn() == 1));
}

int main() {
	setup();
	int result = (testSendsSignalIfThresholdExceeded() && testDoesntSendSignalIfThresholdNotExceeded() && testSetsUpParameters() && testSetsUpNoParameters());
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

