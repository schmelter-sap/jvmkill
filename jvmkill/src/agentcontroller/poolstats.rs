/*
 * Copyright (c) 2015-2018 the original author or authors.
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
use std::collections::HashMap;
use std::io::Write;

pub struct PoolStats {
    pool_hint: HashMap<String, String>
}

impl PoolStats {
    pub fn new() -> Self {
        let mut pool_hint = HashMap::new();
        pool_hint.insert(String::from("Heap memory"), String::from("increase the container size"));
        pool_hint.insert(String::from("Metaspace"), String::from("set -XX:MaxMetaspaceSize to a suitable value"));
        pool_hint.insert(String::from("Compressed Class Space"), String::from("set -XX:CompressedClassSpaceSize to a suitable value"));
        Self {
            pool_hint: pool_hint
        }
    }

    fn print_stats(&self, name: String, stats: Stats, output: &mut Write) {
        let (stats_line, pool_nearly_full) = usage_stats(stats);
        writeln_paced!(output, "   {}: {}", name, stats_line);
        if pool_nearly_full {
            let sw = self.pool_hint.get(&name);
            if let Some(sw_string) = sw {
                writeln_paced!(output, "      Hint: {} is over 95% full. To increase it, {}.", name, sw_string);
            }
        }
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

        let mut output = &mut ::std::io::stdout();

        let mf_class = jni_env.find_class("java/lang/management/ManagementFactory")?;

        writeln_paced!(output, "\nMemory usage:");
        let get_memory_mxbean_method_id = jni_env.get_static_method_id(mf_class, "getMemoryMXBean", "()Ljava/lang/management/MemoryMXBean;")?;
        let memory_mxbean = jni_env.call_static_object_method(mf_class, get_memory_mxbean_method_id)?;

        let memory_mxbean_class = jni_env.get_object_class(memory_mxbean)?;

        let heap_memory_usage_method_id = jni_env.get_method_id(memory_mxbean_class, "getHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;")?;
        let heap_usage = jni_env.call_object_method(memory_mxbean, heap_memory_usage_method_id)?;
        self.print_stats(String::from("Heap memory"), usage_statistics(jni_env, heap_usage)?, &mut output);

        let non_heap_memory_usage_method_id = jni_env.get_method_id(memory_mxbean_class, "getNonHeapMemoryUsage", "()Ljava/lang/management/MemoryUsage;")?;
        let non_heap_usage = jni_env.call_object_method(memory_mxbean, non_heap_memory_usage_method_id)?;
        self.print_stats(String::from("Non-heap memory"), usage_statistics(jni_env, non_heap_usage)?, &mut output);

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
            let name_string = name_cstr.to_string_lossy().into_owned();
            let statistics = usage_statistics(jni_env, usage)?;

            self.print_stats(name_string, statistics, &mut output);
            jni_env.release_string_utf_chars(name, name_utf_chars);
        }
        Ok(())
    }
}

type Stats = (i64, i64, i64, i64);

fn usage_statistics(mut jni_env: ::env::JniEnv, usage: ::jvmti::jobject) -> Result<Stats, ::err::Error> {
    let memory_usage_class = jni_env.get_object_class(usage)?;
    let get_init_method_id = jni_env.get_method_id(memory_usage_class, "getInit", "()J")?;
    let get_used_method_id = jni_env.get_method_id(memory_usage_class, "getUsed", "()J")?;
    let get_committed_method_id = jni_env.get_method_id(memory_usage_class, "getCommitted", "()J")?;
    let get_max_method_id = jni_env.get_method_id(memory_usage_class, "getMax", "()J")?;

    Ok((jni_env.call_long_method(usage, get_init_method_id),
        jni_env.call_long_method(usage, get_used_method_id),
        jni_env.call_long_method(usage, get_committed_method_id),
        jni_env.call_long_method(usage, get_max_method_id)))
}

fn usage_stats(usage: Stats) -> (String, bool) {
    let (init, used, committed, max) = usage;

    let nearly_full = (committed as f32) / (max as f32) >= 0.95;

    ((format!("init {}, used {}, committed {}, max {}", init, used, committed, max), nearly_full))
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_hint_by_default() {
        let mut output = Vec::new();

        let pool_stats = super::PoolStats::new();
        pool_stats.print_stats(String::from("some pool"), (0i64, 1i64, 95i64, 100i64), &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "   some pool: init 0, used 1, committed 95, max 100\n")
    }

    #[test]
    fn no_heap_hint_by_default() {
        let mut output = Vec::new();

        let pool_stats = super::PoolStats::new();
        pool_stats.print_stats(String::from("Heap memory"), (0i64, 1i64, 94i64, 100i64), &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "   Heap memory: init 0, used 1, committed 94, max 100\n")
    }

    #[test]
    fn heap_hint_when_nearly_full() {
        let mut output = Vec::new();

        let pool_stats = super::PoolStats::new();
        pool_stats.print_stats(String::from("Heap memory"), (0i64, 1i64, 95i64, 100i64), &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "   Heap memory: init 0, used 1, committed 95, max 100\n      Hint: Heap memory is over 95% full. To increase it, increase the container size.\n")
    }

    #[test]
    fn metaspace_hint_when_nearly_full() {
        let mut output = Vec::new();

        let pool_stats = super::PoolStats::new();
        pool_stats.print_stats(String::from("Metaspace"), (0i64, 1i64, 95i64, 100i64), &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "   Metaspace: init 0, used 1, committed 95, max 100\n      Hint: Metaspace is over 95% full. To increase it, set -XX:MaxMetaspaceSize to a suitable value.\n")
    }

    #[test]
    fn compressed_class_space_hint_when_nearly_full() {
        let mut output = Vec::new();

        let pool_stats = super::PoolStats::new();
        pool_stats.print_stats(String::from("Compressed Class Space"), (0i64, 1i64, 95i64, 100i64), &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "   Compressed Class Space: init 0, used 1, committed 95, max 100\n      Hint: Compressed Class Space is over 95% full. To increase it, set -XX:CompressedClassSpaceSize to a suitable value.\n")
    }
}
