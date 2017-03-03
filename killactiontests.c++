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
#include "action.h"
#include "killaction.h"

JNIEnv* mockJNIEnv;

Action *action;
const int moddedSignal = SIGUSR1;
bool sigQuit_sent;

void sig_handler(int signo) {
	if (signo == moddedSignal) {
		sigQuit_sent = 1;
	}
}

void setup() {
	if (signal(moddedSignal, sig_handler) == SIG_ERR) {		
        	exit(EXIT_FAILURE);
	}

    KillAction *killAction = new KillAction();
    killAction->setSignal(moddedSignal);
    action = killAction;
    mockJNIEnv = 0;
}

void teardown() {
	signal(moddedSignal, SIG_DFL);
}


bool testSendsSignal() {
	sigQuit_sent = false;

	action->act(mockJNIEnv, 0);

	if (!sigQuit_sent) {
       fprintf(stdout, "testSendsSignal FAILED\n");
    }
	return sigQuit_sent;

}

int main() {
	setup();
	bool result = testSendsSignal();
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
