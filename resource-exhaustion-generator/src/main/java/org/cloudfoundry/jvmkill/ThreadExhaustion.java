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

import java.util.ArrayList;
import java.util.List;

public final class ThreadExhaustion {

    @SuppressWarnings("InfiniteLoopStatement")
    public static void main(String[] args) throws Exception {
        // this does not equate to the exact number of threads that can be created
        // but does help to limit them. In testing a value of 400 allowed about 60 threads to be created
        // in the loop below.
        set_proc_limit(400);

        List<Thread> threads = new ArrayList<>();

        System.out.println("Exhausting threads");

        for (int i = 0; i < 1000; i++) {
            try {
                Thread t = new Sleeper();
                threads.add(t);
                t.start();
                System.out.print(".");
                System.out.println("\nThreads:" + threads.size());
            } catch (Throwable t) {
                System.err.println(t);
            }
        }

        for (Thread thread : threads) {
            thread.interrupt();
            thread.join();
        }
        System.exit(255);
    }

    // set the NPROC limit to a smaller value (same as using `ulimit -u`
    // This can on some systems help to trigger a resource exhaustion without actually exhausting all the resources
    private static void set_proc_limit(int new_limit) {
        Limiter limiter = LimiterFactory.getInstance();
        limiter.setrlimit(Limiter.RLIMIT_NPROC, new RLimits(new_limit, new_limit));
    }

}
