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

// writeln_paced is similar to writeln except it reduces the risk of loggregator missing some entries
// by sleeping before each write. Also, it panics if the underlying writeln fails.
macro_rules! writeln_paced (
    ($($arg:tt)*) => { {
        use std::{thread, time};
        #[allow(unused_imports)]
        use std::io::Write;

        thread::sleep(time::Duration::from_millis(1));

        writeln!($($arg)*).expect("write failed");
    } }
);
