/*
 * Copyright 2015-2022 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.cloudfoundry.jvmkill;

import com.sun.jna.Library;

public interface Limiter extends Library {
    int RLIMIT_CPU = 0;
    int RLIMIT_FSIZE = 1;
    int RLIMIT_DATA = 2;
    int RLIMIT_STACK = 3;
    int RLIMIT_CORE = 4;
    int RLIMIT_RSS = 5;
    int RLIMIT_NPROC = 6;
    int RLIMIT_NOFILE = 7;
    int RLIMIT_MEMLOCK = 8;
    int RLIMIT_AS = 9;
    int RLIMIT_LOCKS = 10;
    int RLIMIT_SIGPENDING = 11;
    int RLIMIT_MSGQUEUE = 12;
    int RLIMIT_NICE = 13;
    int RLIMIT_RTPRIO = 14;
    int RLIMIT_RTTIME = 15;
    int RLIMIT_RLIM_NLIMITS = 16;

    int getrlimit(int resource, RLimits rlim);
    int setrlimit(int resource, RLimits rlim);
}

