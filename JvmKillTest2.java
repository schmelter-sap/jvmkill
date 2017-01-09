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

public final class JvmKillTest2
{
    private static final int MAXNUM = 204800;

    @SuppressWarnings("InfiniteLoopStatement")
    public static void main(String[] args)
            throws Exception
    {
        System.out.println("triggering OutOfMemoryError due to thread exhaustion...");
        List<Thread> list = new ArrayList<>();
        try {
            for (int n = 0; n < MAXNUM; ++n) {
                final int num = n;
                Thread t = new Thread(){
                    private final String name = "Thread number " + num;
                    @Override
                    public void run() {
                        try {
                            Thread.sleep(100000);
                        } catch (Exception e) {
                            System.err.println(this.name + " interrupted by exception " + e);
                        }
                    }
                };
                list.add(t);
                t.start();
                System.err.print(".");
            }
        }
        catch (Throwable t) {
            System.err.println(t.toString());
        }
        System.out.println("final list size: " + list.size());
    }
}

