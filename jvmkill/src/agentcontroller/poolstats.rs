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

pub struct PoolStats {}

impl PoolStats {
    pub fn new() -> Self {
        Self {}
    }
}

impl ::std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "PoolStats")
    }
}

impl super::Action for PoolStats {
    fn on_oom(&self, mut jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint) -> Result<(), ::err::Error> {
        // Do not attempt to obtain pool stats on thread exhaustion as this fails abruptly.
        const threads_exhausted: ::jvmti::jint = ::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as ::jvmti::jint;
        if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
            return Err(::err::Error::ActionUnavailableOnThreadExhaustion("determine memory usage statistics".to_string()));
        }

        let mf_class = jni_env.find_class("java/lang/management/ManagementFactory")?;
        let get_memory_mxbean_method_id = jni_env.get_static_method_id(mf_class, "getMemoryMXBean", "()Ljava/lang/management/MemoryMXBean;")?;
        let memory_mxbean = jni_env.call_static_object_method(mf_class, get_memory_mxbean_method_id)?;

        let memory_mxbean_class = jni_env.get_object_class(memory_mxbean)?;

        let heap_memory_usage_method_id = jni_env.get_method_id(memory_mxbean_class, "getHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;")?;
        let heap_usage = jni_env.call_object_method(memory_mxbean, heap_memory_usage_method_id)?;

        let non_heap_memory_usage_method_id = jni_env.get_method_id(memory_mxbean_class, "getNonHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;")?;
        let non_heap_usage = jni_env.call_object_method(memory_mxbean, non_heap_memory_usage_method_id)?;

        let heap_usage_stats = usage_stats(jni_env, heap_usage)?;
        let non_heap_usage_stats = usage_stats(jni_env, non_heap_usage)?;
        let output = &mut ::std::io::stdout();
        writeln_paced!(output, "\nMemory usage:\n   Heap memory: {}\n   Non-heap memory: {}", heap_usage_stats, non_heap_usage_stats);

        writeln_paced!(output, "\nMemory pool usage:");

        let get_mem_pool_mxbeans_method_id = jni_env.get_static_method_id(mf_class, "getMemoryPoolMXBeans", "()Ljava/util/List;")?;
        let mem_pool_mxbeans = jni_env.call_static_object_method(mf_class, get_mem_pool_mxbeans_method_id)?;
        let list_class = jni_env.find_class("java/util/List")?;
        let size_method_id = jni_env.get_method_id(list_class, "size", "()I")?;
        let get_method_id = jni_env.get_method_id(list_class, "get", "(I)Ljava/lang/Object;")?;
        let mem_pool_mxbean_class = jni_env.find_class("java/lang/management/MemoryPoolMXBean")?;
        let get_name_method_id = jni_env.get_method_id(mem_pool_mxbean_class, "getName", "()Ljava/lang/String;")?;
        let get_usage_method_id = jni_env.get_method_id(mem_pool_mxbean_class, "getUsage", "()Ljava/lang/management/MemoryUsage;")?;
        let size = jni_env.call_int_method(mem_pool_mxbeans, size_method_id);

        for i in 0..size {
            let pool_mxbean = jni_env.call_object_method_with_int(mem_pool_mxbeans, get_method_id, i)?;
            let name = jni_env.call_object_method(pool_mxbean, get_name_method_id)? as ::jvmti::jstring;
            let usage = jni_env.call_object_method(pool_mxbean, get_usage_method_id)?;
            let (name_utf_chars, name_cstr) = jni_env.get_string_utf_chars(name);
            let stats = usage_stats(jni_env, usage)?;
            writeln_paced!(output, "   {}: {}", name_cstr.to_string_lossy().into_owned(), stats);
            jni_env.release_string_utf_chars(name, name_utf_chars);
        }
        Ok(())
    }
}

fn usage_stats(mut jni_env: ::env::JniEnv, usage: ::jvmti::jobject) -> Result<String, ::err::Error> {
    let memory_usage_class = jni_env.get_object_class(usage)?;
    let get_init_method_id = jni_env.get_method_id(memory_usage_class, "getInit", "()J")?;
    let get_used_method_id = jni_env.get_method_id(memory_usage_class, "getUsed", "()J")?;
    let get_committed_method_id = jni_env.get_method_id(memory_usage_class, "getCommitted", "()J")?;
    let get_max_method_id = jni_env.get_method_id(memory_usage_class, "getMax", "()J")?;

    Ok(format!("init {}, used {}, committed {}, max {}",
               jni_env.call_long_method(usage, get_init_method_id),
               jni_env.call_long_method(usage, get_used_method_id),
               jni_env.call_long_method(usage, get_committed_method_id),
               jni_env.call_long_method(usage, get_max_method_id)))
}
