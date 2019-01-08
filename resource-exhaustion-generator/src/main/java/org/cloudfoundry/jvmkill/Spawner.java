/*
 * Copyright 2015-2019 the original author or authors.
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

import java.util.concurrent.CountDownLatch;

final class Spawner extends Thread {

    private final CountDownLatch latch;

    Spawner(CountDownLatch latch) {
        this.latch = latch;
    }

    @Override
    @SuppressWarnings("InfiniteLoopStatement")
    public void run() {
        try {
            this.latch.await();
        } catch (InterruptedException e) {
            // suppress
        }

        System.out.println("Exhausting threads");

        for (; ; ) {
            try {
                new Sleeper().start();
                System.out.print(".");
            } catch (Throwable t) {
                // suppress
            }
        }
    }

}
