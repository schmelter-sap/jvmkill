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

#ifndef threshold_h
#define threshold_h

#include "heuristic.h"

class Threshold: public Heuristic
{
public:
   Threshold(const char *options);

   bool onOOM();

   void kill();

   int getCount_Threshold();

   int getTime_Threshold();

private:
   // circular buffer containing the timestamps of up to count_threshold + 1 OOMs
   long *events; 
   int eventIndex;

   int count_threshold;
   int time_threshold;  // seconds

   void addEvent();
   int countEvents();
   long getMillisLimit();
};

static inline Heuristic* createHeuristic(char *options) {
    return new Threshold(options);
}

#endif // threshold_h