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
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;

public final class JvmKillTestThreadsParallel {
	static final int PARALLEL_SPAWNING = 20;
	static CountDownLatch startSignal;
	static final Runnable spawner = new Runnable() {
		public void run(){
			try {
			startSignal.await();
			} catch (InterruptedException e) {}
			while (true) 
				try {
			                System.out.print("*");
	        			new Thread(spawned).start();
				} catch(Exception e) {}
		}
	};
	static final Runnable spawned = new Runnable() {
		public void run(){
			try {
			Thread.currentThread().sleep(Integer.MAX_VALUE);
			} catch(Exception e) {}
		}
	};
    @SuppressWarnings("InfiniteLoopStatement")
    public static void main(String[] args) throws Exception {
	startSignal = new CountDownLatch(PARALLEL_SPAWNING);
        System.out.print("Spawning Parallel Thread Spawner Threads:");
        for (int i=0;i<PARALLEL_SPAWNING;i++) {
	    try {
	        new Thread(spawner).start();
                System.out.print(".");
		startSignal.countDown();
	    } catch (Throwable t) {
                System.out.println(t.toString());
            }

        }
     }
}
