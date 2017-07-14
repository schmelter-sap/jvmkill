/*
 * Copyright (c) 2015-2017 the original author or authors.
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

#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/jvmti_bindings.rs"));

impl Default for jvmtiCapabilities {
    fn default() -> Self {
        Self {
            _bitfield_1: [0; 4],
            _bitfield_2: [0; 2],
            _bitfield_3: [0; 2],
            _bitfield_4: [0; 2],
            __bindgen_align: [],
        }
    }
}

impl Default for jvmtiHeapCallbacks {
    fn default() -> jvmtiHeapCallbacks {
        Self {
            heap_iteration_callback: None,
            heap_reference_callback: None,
            primitive_field_callback: None,
            array_primitive_value_callback: None,
            string_primitive_value_callback: None,
            reserved5: None,
            reserved6: None,
            reserved7: None,
            reserved8: None,
            reserved9: None,
            reserved10: None,
            reserved11: None,
            reserved12: None,
            reserved13: None,
            reserved14: None,
            reserved15: None,
        }
    }
}

impl Default for jvmtiEventCallbacks {
    fn default() -> jvmtiEventCallbacks {
        Self {
            VMInit: None,
            VMDeath: None,
            ThreadStart: None,
            ThreadEnd: None,
            ClassFileLoadHook: None,
            ClassLoad: None,
            ClassPrepare: None,
            VMStart: None,
            Exception: None,
            ExceptionCatch: None,
            SingleStep: None,
            FramePop: None,
            Breakpoint: None,
            FieldAccess: None,
            FieldModification: None,
            MethodEntry: None,
            MethodExit: None,
            NativeMethodBind: None,
            CompiledMethodLoad: None,
            CompiledMethodUnload: None,
            DynamicCodeGenerated: None,
            DataDumpRequest: None,
            reserved72: None,
            MonitorWait: None,
            MonitorWaited: None,
            MonitorContendedEnter: None,
            MonitorContendedEntered: None,
            reserved77: None,
            reserved78: None,
            reserved79: None,
            ResourceExhausted: None,
            GarbageCollectionStart: None,
            GarbageCollectionFinish: None,
            ObjectFree: None,
            VMObjectAlloc: None
        }
    }
}
