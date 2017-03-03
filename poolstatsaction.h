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

#ifndef poolstatsaction_h
#define poolstatsaction_h

#include "action.h"

#include <string>

class PoolStatsAction: public Action
{
public:
   PoolStatsAction();

   virtual ~PoolStatsAction();

   void act(JNIEnv* jniEnv, jint resourceExhaustionFlags);

private:
   std::string usageStats(JNIEnv* jniEnv, jobject usage);
};

#endif // poolstatsaction_h
