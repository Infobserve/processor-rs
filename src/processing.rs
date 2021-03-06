#![allow(dead_code)]

use std::{fs, str, thread, sync::Arc, time, fmt};
use log::{info, warn, error};

use yara::{Compiler, Rules, Rule, YaraError};
use crossbeam_channel::{Sender, Receiver};
use anyhow::Result;

use crate::utils::{pluralize, rec_get_files_by_ext};
use crate::errors::ConfigurationError;
use crate::entities::{Event, FlatMatch, ProcessedEvent};

/// Spawns `num_processors` threads each of which continuously pops from the read-end of a crossbeam channel,
/// processes the events, enriches matching ones with additional information (e.g. the matched string) and pushes them
/// to the write-end of another crossbeam channel -- These are later stored in Postgres by another thread
/// 
/// # Arguments
/// 
/// * feed_recvr - The read-end of a crossbeam channel. While the write-end is not dropped, all threads hang
///                until an event is available (only one thread processes each event)
/// * load_sendr - The write-end of a crossbeam channel. After processing events, it turns them into `ProcessedEvent` objects
///                (the initial event (`Event`) + information on the match (`FlatMatch`)) and pushes them into the channel
/// * yara_dir - The fully qualified path to the root of a yara rule directory. This directory will be recursively walked and
///              all Yara rule files (*.yar) will be loaded to the processor
/// * num_processors - The number of threads to spawn. Each will hang on `feed_recvr` waiting for new messages (events)
/// 
/// # Return
/// Returns a vector of join handles that can be used to join the threads after the feed crossbeam channel's write-end
/// has been dropped. Notice that since all processing workers handle errors themselves (if they are salvageable),
/// the returned join handles carry no information when their respective threads are joined upon
/// 
/// # Example
/// 
/// ```
/// use processing::start_processors;
/// use entities::Event;
/// 
/// let (feed_sendr, feed_recvr) = crossbeam_channel::unbounded();
/// let (load_sendr, load_recvr) = crossbeam_channel::unbounded();
///
/// let handles: Vec<JoinHandle<()>> = start_processors(&feed_recevr, &load_sendr, "path/to/yara/dir", 3);
///
/// assert_eq!(handles.len(), 3);
/// // Note that it's the responsibility of the thread that created the crossbeam channels to drop them as well
/// // let e = Event::new(...);
/// // feed_sendr.send(e);
///
/// drop(feed_sendr);
/// drop(load_sendr);
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// ```
pub fn start_processors(
    feed_recvr: &Receiver<Event>,
    load_sendr: &Sender<ProcessedEvent>,
    yara_dir: &str,
    num_processors: usize
) -> Vec<thread::JoinHandle<Result<Stats>>> {
    let yara_dir_arc = Arc::new(yara_dir.to_owned());
    let mut p_handles: Vec<thread::JoinHandle<Result<Stats>>> = Vec::new();

    info!("Spawning {}", pluralize(num_processors as i64, "processor"));
    for _ in 0..num_processors {
        p_handles.push(process_forever(feed_recvr, load_sendr, &yara_dir_arc));
    }

    p_handles
}

/// Given the read-end of a crossbeam channel and a Yara rule directory,
/// spawns a new thread which continuously reads events from the channel and passes them
/// through the processor.
/// Events that match one or more rules are then persisted
/// to the DB (see database::loader::DbLoader)
/// 
/// Returns the join handle for the newly spawned thread
#[allow(clippy::rc_buffer)]
fn process_forever(
    feed_recvr: &Receiver<Event>,
    load_sendr: &Sender<ProcessedEvent>,
    yara_dir_arc: &Arc<String>
) -> thread::JoinHandle<Result<Stats>> {
    let rx = Receiver::clone(feed_recvr);
    let sx = Sender::clone(load_sendr);
    let yara_dir = Arc::clone(&yara_dir_arc);

    thread::spawn(move || {
        let mut stats = Stats::new();

        let p = Processor::from_dir(&yara_dir)?;

        for message in rx {
            let start = time::Instant::now();
            stats.inc_events();
            match p.process(message.raw_content()) {
                Ok(m) => {
                    if !m.is_empty() {
                        stats.inc_matches();
                        if let Err(e) = sx.send(ProcessedEvent(message, m)) {
                            error!("Failed to send processed event: {}", e);
                            stats.inc_failures();
                        }
                    } else {
                        warn!("Zero length match? {:?}", message);
                    }
                }
                Err(e) => println!("Whoops: {:?}", e)
            }
            stats.add_duration(start.elapsed());
        }

        Ok(stats)
    })
}

struct Processor {
    engine: Rules
}

impl Processor {
    /// Constructs a Processor object whose rules have been loaded recursively
    /// from a directory
    ///
    /// # Arguments
    ///
    /// * `rule_root` - The root directory under which `.yar` files will be found
    ///
    /// # Examples
    ///
    /// ```
    /// let p: Processor = Processor::from_dir("yara-rules/");
    /// ```
    ///
    /// # Errors
    ///
    /// `crate::errors::NoYaraRulesError` - When no `.yar` files can be found under `rule_root`
    fn from_dir(rule_root: &str) -> Result<Processor> {
        let rule_files = rec_get_files_by_ext(rule_root, "yar");

        if rule_files.is_empty() {
            error!("Found no .yar files under {}. Refusing to continue", rule_root);
            return Err(ConfigurationError::NoYaraRulesError.into());
        }

        Processor::with_rule_files(rule_files)
    }

    /// Constructs a Processor object whose rules have been loaded by
    /// the contents of the provided files
    /// Largely works the same as `Processor::from_dir`, but each file must
    /// be passed explicitly
    fn with_rule_files(filenames: Vec<String>) -> Result<Processor> {
        let mut rules: Vec<String> = Vec::new();
        for filename in filenames.into_iter() {
            rules.push(fs::read_to_string(filename)?);
        }

        Processor::with_rules(rules)
    }

    /// Constructs a Processor object from a string representing a Yara rule
    ///
    /// # Arguments
    ///
    /// * `rule` - The Yara rule
    #[allow(dead_code)]
    fn with_rule_str(rule: &str) -> Result<Processor> {
        Processor::with_rules(vec![rule.to_string()])
    }

    /// Constructs a Processor object from a vector of strings, each of which
    /// represents a Yara rule
    ///
    /// # Arguments
    ///
    /// * `rules` - A vector of Yara rule strings
    fn with_rules(rules: Vec<String>) -> Result<Processor> {
        let mut compiler = Compiler::new()?;

        for rule in rules.into_iter() {
            compiler.add_rules_str(&rule)?;
        }

        let engine = compiler.compile_rules()?;
        Ok(Processor { engine })
    }

    /// Given a string, tries to match the compiled Yara rules against it
    /// Returns the matches as a vector of `FlatMatch` objects
    ///
    /// # Arguments
    ///
    /// * `filestr` - The string against which the Yara matcher will run
    ///
    /// # Examples
    /// ```
    /// let p = Processor::with_rule_files("yara-rules/MyPassword.yar");
    /// let matches: Vec<FlatMatch> = p.process("password: HelloWorld").unwrap();
    /// for m in matches {
    ///     m.rule_name(); // "MyPassword"
    ///     m.tags(); // ["my", "matched", "rule", "tags"]
    ///     m.data(); // ["HelloWorld"]
    /// }
    /// ```
    fn process(&self, filestr: &str) -> Result<Vec<FlatMatch>, YaraError> {
        let rules: Vec<Rule> = self.engine.scan_mem(filestr.as_bytes(), 10)?;
        Ok(FlatMatch::from_rules(rules))
    }
}

pub struct Stats {
    overall_proc_time: time::Duration,
    num_events: u32,
    num_matches: u32,
    num_failures: u32
}

impl Stats {
    fn new() -> Self {
        Self {
            overall_proc_time: time::Duration::from_secs(0),
            num_events: 0,
            num_matches: 0,
            num_failures: 0
        }
    }

    fn add_duration(&mut self, elapsed: time::Duration) {
        self.overall_proc_time += elapsed;
    }

    fn inc_events(&mut self) {
        self.num_events += 1;
    }

    fn inc_matches(&mut self) {
        self.num_matches += 1;
    }

    fn inc_failures(&mut self) {
        self.num_failures += 1;
    }

    pub fn overall_proc_time(&self) -> time::Duration {
        self.overall_proc_time
    }

    pub fn avg_proc_time(&self) -> time::Duration {
        if self.num_events == 0 {
            return time::Duration::from_secs(0);
        }

        self.overall_proc_time / self.num_events
    }

    pub fn num_events(&self) -> u32 {
        self.num_events
    }

    pub fn num_matches(&self) -> u32 {
        self.num_matches
    }

    pub fn num_failures(&self) -> u32 {
        self.num_failures
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  Overall time spent processing: {}\n  Average time per event processing: {}\n  Events processed: {}\n  Events matched: {}",
            self.overall_proc_time().as_millis(),
            self.avg_proc_time().as_millis(),
            self.num_events(),
            self.num_matches()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password_rule() -> String {
        String::from(r#"
        rule MyPass
        {
            meta:
                name = "My Pass"

            strings:
                $a = /pw:.+/

            condition:
                $a
        }
        "#)
    }

    fn processor() -> Processor {
        Processor::with_rule_str(&password_rule()).unwrap()
    }

    #[test]
    fn processor_does_not_blow_up() {
        processor();
    }

    #[test]
    #[should_panic]
    fn processor_blows_up_with_bad_rule() {
        Processor::with_rule_str("Bad Rule").unwrap();
    }

    #[test]
    fn process_does_not_blow_up() {
        let p = processor();
        p.process(&"foo").unwrap();
    }

    #[test]
    fn process_returns_correct_data() {
        let p = processor();
        let matches = p.process(&"pw: helloworld").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_name(), String::from("default::MyPass"));
        assert_eq!(matches[0].tags().len(), 0);
        assert_eq!(*matches[0].data()[0], String::from("pw: helloworld"));
    }

    #[test]
    fn stats_start_from_zero() {
        let s = Stats::new();
        assert_eq!(s.overall_proc_time().as_millis(), 0);
        assert_eq!(s.num_events(), 0);
        assert_eq!(s.num_matches(), 0);
    }

    #[test]
    fn stats_count_events_correctly() {
        let mut s = Stats::new();
        s.inc_events();
        assert_eq!(s.num_events(), 1);
    }

    #[test]
    fn stats_count_matches_correctly() {
        let mut s = Stats::new();
        s.inc_matches();
        assert_eq!(s.num_matches(), 1);
    }

    #[test]
    fn stats_return_zero_avg_time_when_no_events_are_processed() {
        let s = Stats::new();
        assert_eq!(s.avg_proc_time().as_millis(), 0);
    }

    #[test]
    fn stats_count_overall_duration_correctly() {
        let mut s = Stats::new();
        let d1 = time::Duration::from_millis(10);
        s.add_duration(d1);
        let d2 = time::Duration::from_secs(1);
        s.add_duration(d2);
        assert_eq!(s.overall_proc_time, time::Duration::from_millis(1010));
    }

    #[test]
    fn stats_calculates_avg_correctly() {
        let mut s = Stats::new();
        let d = time::Duration::from_secs(1);
        s.add_duration(d);
        for _ in 0..5 {
            s.inc_events();
        }

        assert_eq!(s.avg_proc_time().as_millis(), 200);
    }
}
